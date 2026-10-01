# Roadmap — étapes d'implémentation par phases

Stratégie : **incrémentale, bottom-up, pas de big-bang**. On monte la pile depuis le
stockage vers le moteur, chaque phase livre quelque chose de testable en isolation.

Légende : **DoD** = Definition of Done (critère de fin de phase).

---

## Phase 0 — Socle & contrat  ·  *partiellement livré (scaffold)*

**Objectif.** Un squelette qui compile, le contrat gRPC figé, les types de base, et un
harnais de tests de conformité.

**Crates.** `common`, `proto`, `model`, `gateway`, `broker`, `testkit`, `xtask`.

**Étapes.**
1. ✅ Workspace Cargo, lints, édition 2024, DAG de crates, daemon `broker` qui démarre.
2. ✅ `proto/gateway.proto` (contrat client, package `workflow.v1`).
3. ⬜ Câbler `tonic-build` dans `proto/build.rs` → générer les types gRPC ; exposer
   `proto::gateway`.
4. ⬜ Définir dans `model` les types fondateurs : `Record`, `Intent`, `ValueType`,
   clés/positions, (dé)sérialisation (`serde` + un encodage binaire interne choisi).
5. ⬜ Implémenter `gateway` : serveur `tonic` qui répond à `Topology` et accepte
   `DeployProcess` / `CreateProcessInstance` contre un **état en mémoire** (stub).
6. ⬜ `testkit` : horloge déterministe + fakes ; premier test d'intégration
   bout-en-bout gateway↔stub (**M-INTEGRATION-TESTS**).
7. ⬜ `xtask codegen` : régénère les bindings proto.

**Dépendances externes introduites.** `tonic`, `prost`, `tonic-build`, `rmp-serde`.

**DoD.** `cargo test` vert ; un client gRPC obtient une réponse à `Topology` et peut
déposer/créer une instance « en mémoire ». Contrat gRPC gelé.

**Risques.** Choix de l'encodage interne des records (impacte tout le reste).

---

## Phase 1 — Couche stockage

**Objectif.** Un log durable et un store d'état, testables hors cluster.

**Crates.** `journal`, `state`.

**Étapes.**
1. ⬜ `journal` : écriture append-only en segments, index des positions, lecture
   séquentielle, troncature, fsync/durabilité, rotation de segments.
2. ⬜ `state` : abstraction au-dessus de RocksDB (familles de colonnes typées,
   transactions, itérateurs).
3. ⬜ `state::snapshot` : prise de snapshot cohérente de l'état + métadonnées
   (position du log correspondante).
4. ⬜ Compaction : lier snapshot ↔ troncature du journal.
5. ⬜ Tests : propriété « rejouer le log reconstruit exactement l'état » ; crash-recovery
   (réouverture après arrêt brutal simulé).

**Dépendances externes introduites.** `rocksdb` (nécessite un toolchain C/clang au
build — déjà géré côté Docker/WSL).

**DoD.** Écrire/relire/tronquer un log ; prendre/restaurer un snapshot ; tests de
recovery verts.

**Risques.** Durabilité (ordre des fsync) ; coût du build de `rocksdb`.

---

## Phase 2 — Consensus & cluster

**Objectif.** Répliquer une partition sur plusieurs nœuds avec un leader élu.

**Crates.** `transport`, `cluster`.

**Étapes.**
1. ⬜ Spike `openraft` : POC log + snapshot + élection sur 3 nœuds en mémoire.
2. ⬜ `transport` : messagerie framée entre brokers (tokio + rustls), (dé)sérialisation
   des messages Raft.
3. ⬜ `cluster` : intégrer `openraft` derrière une interface maison ; brancher le
   `RaftLogStorage` sur `journal` et le `RaftStateMachine` sur `state`/`engine`.
4. ⬜ Membership : découverte des nœuds, join/leave, N partitions par nœud.
5. ⬜ Tests : élection, réplication, perte de leader, rattrapage d'un follower via
   snapshot.

**Dépendances externes introduites.** `openraft`, `rustls`.

**DoD.** Un cluster 3 nœuds réplique une partition, survit à la perte du leader, et un
nouveau follower se resynchronise.

**Risques.** ⚠️ Le plus gros risque du projet : exactitude du consensus = intégrité des
données. Intégration snapshot/membership délicate.

---

## Phase 3 — Moteur mono-partition

**Objectif.** Exécuter réellement des processus sur une partition.

**Crates.** `engine` (dépend de `journal` pour lire le log et de `state`), `feel`.

**Étapes.**
1. ⬜ **Décision FEEL** (début de phase) : implémentation Rust vs binding vs
   sous-ensemble. Débloque `feel`.
2. ⬜ `engine::processor` : boucle du stream processor déterministe (consomme le log,
   applique, écrit état + records de suivi) ; contrôle strict des sources de
   non-déterminisme (horloge via `testkit`, pas de flottants/itération non ordonnée).
3. ⬜ Comportements BPMN d'un **sous-ensemble minimal** : start/end event, séquence,
   service task, exclusive gateway.
4. ⬜ Contextes métier : `process` (instances), `job`, `deployment`, `variable`.
5. ⬜ `feel` : évaluer les expressions de variables/conditions du sous-ensemble.
6. ⬜ Élargir : message/timer/incident, parallel/event-based gateway, sous-process.
7. ⬜ Tests : rejeu déterministe (même log → même état), couverture par élément.

**Dépendances externes introduites.** éventuellement un crate de parsing pour `feel`.

**DoD.** Déployer un processus simple (start → service task → end), créer une instance,
activer/compléter un job, la mener à terme ; rejeu déterministe prouvé par test.

**Risques.** FEEL ; garantir le déterminisme à grande échelle.

---

## Phase 4 — Gateway complet, client & exporters

**Objectif.** Exposer toute l'API aux clients et sortir les données.

**Crates.** `gateway`, `client`, `exporter`, `cli`.

**Étapes.**
1. ⬜ `gateway` : implémenter tous les RPC du contrat, router vers le leader de la bonne
   partition, gérer le streaming `ActivateJobs` (long-poll) et le back-pressure.
2. ⬜ `client` : SDK async (job workers : poll + complete/fail) (**M-ASYNC-FN**).
3. ⬜ `exporter` : trait `Exporter` + implémentation (ex. index de recherche) derrière
   une feature (**M-FEATURES-ADDITIVE**) ; suivi du log committé, reprise sur position.
4. ⬜ `cli` : commandes `topology`, `deploy`, `create`, `status`.
5. ⬜ `broker` : assembler cluster + partitions + engine + gateway + exporter ;
   configuration (fichier/env) ; arrêt gracieux.
6. ⬜ Docker : `EXPOSE` du port gRPC + `docker run -p` ; (option) `docker compose` 3
   nœuds.

**Dépendances externes introduites.** client/exporter spécifiques (ex. client HTTP).

**DoD.** Un worker externe se connecte au `broker` conteneurisé, traite des jobs
bout-en-bout ; les records exportés sont visibles dans le système cible.

---

## Phase 5 — Parité, durcissement & performance

**Objectif.** Rendre le moteur crédible en production.

**Crates.** toutes.

**Étapes.**
1. ⬜ Élargir la couverture BPMN (sous-process d'événement, compensation, multi-instance…).
2. ⬜ Chaos testing : pannes réseau, pertes de nœuds, partitions réseau.
3. ⬜ Performance : identifier/profiler le hot path tôt (**M-HOTPATH**), optimiser
   débit (**M-THROUGHPUT**), hasher rapide (**M-FAST-HASHER**), capacités initiales
   (**M-INITIAL-CAPACITY**), points de yield (**M-YIELD-POINTS**) ; benchmarks
   `criterion`.
3. ⬜ Allocateur `mimalloc` activé en prod (**M-MIMALLOC-APPS**), `target-cpu` réglé
   (**M-TARGET-CPU**).
4. ⬜ Durcissement : `cargo deny` (licences/CVE), fuzzing des parsers (model, feel).

**DoD.** Benchmarks publiés, tests de chaos verts, couverture BPMN documentée.

---

## Dépendances transverses entre phases

```
Phase 0 (contrat, types) ─┬─► Phase 1 (stockage) ─► Phase 2 (consensus) ─┐
                          └─────────────────────► Phase 3 (moteur) ◄─────┘
                                                        │
                                   Phase 4 (gateway/client/exporter) ◄────┘
                                                        │
                                              Phase 5 (parité/perf)
```

Phase 3 (moteur mono-partition) peut démarrer **en parallèle** de Phase 2 en utilisant
un log local non répliqué (fourni par `testkit`), puis se brancher sur le vrai cluster.
