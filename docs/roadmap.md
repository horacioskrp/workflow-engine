# Roadmap — étapes d'implémentation par phases

Stratégie : **incrémentale, bottom-up**. On monte depuis le socle et la persistance
vers les capacités métier, chaque phase livre quelque chose de testable en isolation.

Légende : **DoD** = Definition of Done (critère de fin de phase).

---

## Phase 0 — Socle & contrat  ·  *livré*

**Objectif.** Un squelette qui compile, le contrat d'API figé, les types de base, un
harnais de tests, la CI et le flux gitflow.

**Crates.** `kernel`, `contracts`, `api`, `node`, `harness`, `xtask`.

**Étapes.**
1. ✅ Workspace Cargo (capacités), lints, édition 2024, DAG de crates, démon `node`.
2. ✅ `proto/gateway.proto` (contrat client, package `workflow.v1`).
3. ✅ `tonic-build` câblé dans `contracts/build.rs` → types gRPC exposés via
   `contracts::v1` (validé par `docker build` + test `contracts/tests/codegen.rs`).
4. ✅ Processus : flux **gitflow** (`main`/`develop`/`feature/*`, cf.
   [CONTRIBUTING](../CONTRIBUTING.md)) + **CI** GitHub Actions (fmt + clippy `-D warnings`
   + test, avec `protoc`) + premiers tests.
5. ✅ `kernel` : identifiants typés (`NodeId`, `PartitionId`, `Key`) + horloge
   abstraite (`Clock` trait, `Timestamp`, `SystemClock`). La télémétrie = façade
   `tracing` dans les libs, subscriber installé par les binaires.
6. ✅ `api` : service `tonic` implémentant le trait `Gateway` — `Topology` répond
   depuis un **état en mémoire** (cluster 1 nœud / 1 partition), les autres RPC
   renvoient `unimplemented`. `node` sert l'API sur `0.0.0.0:26500`. Test unitaire
   async de `Topology`.
7. ✅ `harness` : horloge déterministe `ManualClock` (implémente `kernel::Clock`) +
   fake `RecordingSink` ; test d'intégration **bout-en-bout** `Topology` client↔serveur
   gRPC (`api/tests/topology_e2e.rs`, **M-INTEGRATION-TESTS**).
8. ✅ `xtask codegen` : régénère les bindings en rebâtissant `contracts`.

**Dépendances externes introduites.** `tonic`, `prost`, `tonic-build`, `tokio-stream`.

**DoD — atteinte.** CI verte ; un client gRPC obtient une réponse à `Topology` (test
e2e). Contrat gRPC gelé. (Le `DeployProcess`/`CreateProcessInstance` en mémoire est
reporté : les capacités réelles arrivent en Phase 3.)

---

## Phase 1 — Persistance  ·  *en cours*

**Objectif.** Un socle de stockage durable (état + historique), testable isolément.

**Crates.** `persistence`.

**Étapes.**
1. ⬜ `persistence::store` : état clé/valeur transactionnel (sur backend embarqué),
   familles typées, itérateurs.
2. 🟡 `persistence::history` : seam `History` (trait) + `Position`/`HistoryEntry` +
   impl **en mémoire** `MemoryHistory` (append monotone, `read_from`, `last_position`),
   testés. **Reste** : backend durable (segments sur disque, fsync, reprise après crash).
3. ⬜ `persistence::snapshot` : snapshot cohérent de l'état + position d'historique
   associée ; compaction liée.
4. ⬜ Tests : « rejouer l'historique reconstruit exactement l'état » ; recovery après
   arrêt brutal simulé.

**Approche incrémentale.** On livre d'abord les abstractions + impl en mémoire
(testables sans toolchain C), puis le backend durable `rocksdb`/segments. L'impl mémoire
sert aussi de fake pour les phases suivantes (cf. `harness`).

**Dépendances externes introduites.** `rocksdb` (toolchain C au build — géré via Docker/WSL).

**DoD.** Écrire/relire l'historique ; prendre/restaurer un snapshot ; tests de recovery verts.

---

## Phase 2 — Coordination & cluster

**Objectif.** Répliquer l'état persisté sur plusieurs nodes avec un primaire élu.

**Crates.** `coordination` (+ transport interne).

**Étapes.**
1. ⬜ Spike `openraft` : POC log + snapshot + élection sur 3 nodes.
2. ⬜ Messagerie inter-nodes (tokio + rustls) pour les échanges de réplication.
3. ⬜ `coordination::replication` : brancher le stockage de log/snapshot de `openraft`
   sur `persistence`.
4. ⬜ `coordination::membership` : découverte, join/leave, attribution des tranches de
   charge par node.
5. ⬜ Tests : élection, réplication, perte du primaire, rattrapage par snapshot.

**Dépendances externes introduites.** `openraft`, `rustls`.

**DoD.** Un cluster 3 nodes réplique une tranche, survit à la perte du primaire, et un
nouveau réplica se resynchronise.

**Risques.** ⚠️ Risque n°1 du projet : l'exactitude de la coordination = intégrité des
données.

---

## Phase 3 — Capacités métier

**Objectif.** Exécuter réellement des processus.

**Crates.** `workflow`, `expr`, `scheduling`, `tasks`, `messaging`.

**Étapes.**
1. ⬜ **Décision `expr`** (début de phase) : implémentation vs binding vs sous-ensemble.
2. ⬜ `workflow::execution` : moteur d'exécution **déterministe** (lit l'historique,
   applique, écrit état + événements) ; contrôle strict du non-déterminisme (horloge
   via `harness`).
3. ⬜ `workflow::model` + `activity` : sous-ensemble minimal (début/fin, séquence,
   tâche de service, branchement exclusif).
4. ⬜ `tasks` : création de work items + activation par les workers.
5. ⬜ `scheduling` : timers/échéances déclenchant la reprise d'exécution.
6. ⬜ `messaging` : abonnements + corrélation de messages.
7. ⬜ `expr` : évaluer conditions et expressions de variables.
8. ⬜ Élargir : branchements parallèle/événementiel, sous-processus, incidents.
9. ⬜ Tests : rejeu déterministe (même historique → même état) ; couverture par élément.

**DoD.** Déployer un processus simple, créer une instance, activer/compléter une tâche,
la mener à terme ; rejeu déterministe prouvé par test.

**Risques.** `expr` ; garantir le déterminisme à grande échelle.

---

## Phase 4 — API complète, client, CLI & feed

**Objectif.** Exposer toute l'API et sortir les données.

**Crates.** `api`, `sdk`, `ctl`, `feed`.

**Étapes.**
1. ⬜ `api` : tous les RPC du contrat, routage vers le node responsable, streaming
   d'activation de tâches (long-poll), back-pressure.
2. ⬜ `sdk` : client async (workers : poll + complete/fail) (**M-ASYNC-FN**).
3. ⬜ `feed` : trait `Sink` + implémentation (ex. index de recherche) derrière une
   feature (**M-FEATURES-ADDITIVE**) ; reprise sur position.
4. ⬜ `ctl` : commandes `topology`, `deploy`, `create`, `status`.
5. ⬜ `node` : assembler coordination + capacités + api + feed ; configuration ; arrêt
   gracieux.
6. ⬜ Docker : `EXPOSE` du port gRPC + `docker run -p` ; (option) compose multi-nodes.

**DoD.** Un worker externe se connecte au `node` conteneurisé, traite des tâches
bout-en-bout ; l'historique exporté est visible dans le système cible.

---

## Phase 5 — Parité, durcissement & performance

**Crates.** toutes.

**Étapes.**
1. ⬜ Élargir la couverture du modèle de processus.
2. ⬜ Chaos testing : pannes réseau, pertes de nodes, partitions réseau.
3. ⬜ Performance : hot path tôt (**M-HOTPATH**), débit (**M-THROUGHPUT**), hasher
   rapide (**M-FAST-HASHER**), capacités initiales (**M-INITIAL-CAPACITY**), points de
   yield (**M-YIELD-POINTS**) ; benchmarks `criterion`.
4. ⬜ `mimalloc` en prod (**M-MIMALLOC-APPS**), `target-cpu` réglé (**M-TARGET-CPU**).
5. ⬜ `cargo deny` (licences/CVE), fuzzing des parsers (`expr`, modèle).

**DoD.** Benchmarks publiés, tests de chaos verts, couverture documentée.

---

## Dépendances entre phases

```
Phase 0 (contrat, socle) ─► Phase 1 (persistance) ─► Phase 2 (coordination) ─┐
                            └──────────────────────► Phase 3 (capacités) ◄────┘
                                                            │
                                   Phase 4 (api/sdk/ctl/feed) ◄──┘
                                                            │
                                                  Phase 5 (parité/perf)
```

Phase 3 (capacités) peut démarrer **en parallèle** de Phase 2 avec une persistance
locale non répliquée (fournie par `harness`), puis se brancher sur le vrai cluster.
