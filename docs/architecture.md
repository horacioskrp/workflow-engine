# Architecture — constitution & gros briques

## 1. Vision

Un moteur de workflow distribué : les clients déposent des définitions de processus
et pilotent des instances via une **API gRPC** ; l'état vit dans un **log d'événements
répliqué** + un **store clé/valeur embarqué** (pas de base relationnelle) ; la
scalabilité et la tolérance aux pannes viennent de **partitions répliquées par Raft**.

Principe fondateur : **M-RUST-SHAPED** — on résout les problèmes avec des patterns
Rust, on ne transpose pas une architecture d'un autre langage.

## 2. Les couches

```
┌─────────────────────────────────────────────────────────────┐
│ APP        broker (daemon)         cli (admin)                │
├─────────────────────────────────────────────────────────────┤
│ BORDURE    gateway (gRPC)          client (SDK)               │
├─────────────────────────────────────────────────────────────┤
│ MOTEUR     engine (stream proc.)   exporter                   │
├─────────────────────────────────────────────────────────────┤
│ CLUSTER    cluster (Raft)          transport                  │
├─────────────────────────────────────────────────────────────┤
│ STOCKAGE   journal (log)           state (KV + snapshots)     │
├─────────────────────────────────────────────────────────────┤
│ FONDATION  common   proto   model   feel                      │
└─────────────────────────────────────────────────────────────┘
          OUTILLAGE : xtask (automatisation)  ·  testkit (tests)
```

## 3. Les gros briques (15 crates)

| Crate | Rôle | Techno clé | Phase |
|---|---|---|:--:|
| `common` | Primitives partagées : identifiants, horloge, scaffolding d'erreur | std | 0 |
| `proto` | Types gRPC/protobuf générés depuis `proto/gateway.proto` | tonic-build, prost | 0 |
| `model` | Records, intents, modèle de processus (données pures) | serde, rmp-serde | 0 |
| `feel` | Langage d'expression embarqué : parser + évaluateur | — (à écrire) | 3 |
| `journal` | Log append-only répliqué, en segments sur disque | — | 1 |
| `state` | Store clé/valeur + colonnes + snapshots | rocksdb | 1 |
| `transport` | Transport réseau nœud-à-nœud | tokio, rustls | 2 |
| `cluster` | Consensus Raft + membership | openraft | 2 |
| `engine` | Stream processor déterministe + comportements de processus | — | 3 |
| `exporter` | Trait Exporter + exporters intégrés | serde | 4 |
| `gateway` | Serveur gRPC, routage vers les partitions | tonic, tokio | 4 |
| `client` | SDK client Rust | tonic | 4 |
| `broker` | Daemon : bootstrap et assemblage de tout | tokio, anyhow, mimalloc | 0→ |
| `cli` | CLI admin/client | anyhow | 4 |
| `testkit` | Utilitaires de test : horloge déterministe, fakes | — | 0→ |

(+ `xtask` : automatisation de build/codegen, hors graphe applicatif.)

## 4. Graphe de dépendances (sens unique, sans cycle)

```
common ──┬─ model ──┬─ feel
         │          └─ exporter
         ├─ state                  (store KV générique — pas de dép. model, cf. zb-db)
         ├─ journal
         ├─ transport ─┐
         │             └─ cluster
         └─ proto

engine   ← common, model, journal, state, feel   (le stream processor LIT le log)
gateway  ← common, proto, model
client   ← common, proto          ;   cli ← client, proto
broker   ← common, journal, state, transport, cluster, engine, exporter, gateway
```

Respecte **M-CRATES-FLAT-FOLDER** (crates plates sous `crates/`) et
**M-CRATES-IN-WORKSPACE** (deps internes via `[workspace.dependencies]`).

> **Fidélité au source.** Vérifié contre les `pom.xml` réels de `zeebe-develop` :
> `engine` dépend de `logstreams` (→ `journal`) et `zb-db` est un store générique
> (→ `state` sans dép. `model`, les vues typées vivant dans `engine::state`). Le
> module `monitor` (monitoring Zeebe) n'est pas une brique du moteur : il relève du
> transverse observabilité (voir §6).

## 5. Modèle d'exécution runtime (comment ça s'assemble)

Le `broker` est un nœud du cluster. À l'exécution :

1. **Membership & partitions.** `cluster` établit l'appartenance au cluster et héberge
   N partitions. Chaque partition est un groupe Raft répliqué sur plusieurs brokers ;
   l'un est leader.
2. **Une partition = log + état + moteur.** Pour chaque partition dont ce broker est
   réplica :
   - `journal` détient le log répliqué (écrit par le leader via Raft, répliqué aux
     followers).
   - `engine` fait tourner un **stream processor** : il consomme les records du log
     **dans l'ordre, de façon déterministe**, applique les comportements de processus,
     et écrit l'état résultant dans `state` + de nouveaux records (commandes de suivi)
     dans le log.
   - `state` matérialise l'état (instances, jobs, timers, abonnements message) et
     produit des `snapshots` pour compacter le log.
3. **Entrée client.** `gateway` (gRPC) reçoit les appels, les transforme en commandes
   et les route vers le **leader de la bonne partition** (via `transport`/`cluster`).
   Les réponses reviennent quand le record est committé et traité.
4. **Sortie.** `exporter` suit le log committé et pousse les records vers l'extérieur
   (index de recherche, etc.) pour le monitoring/analytique.

> Le déterminisme du point 2 est **la** propriété centrale : rejouer le même log doit
> reproduire exactement le même état sur chaque réplica et après redémarrage.

## 6. Décisions structurantes

- **Compatibilité API-only.** On fige le contrat gRPC (`proto/gateway.proto`) ; les
  formats internes (log, snapshot, protocole inter-nœuds) sont les nôtres. Pas de
  compatibilité on-disk avec un système existant → bien plus réaliste.
- **Erreurs** : chaque crate bibliothèque expose un `Error` struct canonique avec
  `Backtrace` (**M-ERRORS-CANONICAL-STRUCTS**) ; les binaires utilisent `anyhow`
  (**M-APP-ERROR**). Conversions via `From` (**M-FROM-ERROR**).
- **Sûreté** : `unsafe` interdit au niveau workspace (**M-UNSAFE**) ; une crate qui en
  aurait besoin (FFI) relâche localement.
- **Observabilité** : `tracing` partout, logs structurés (**M-LOG-STRUCTURED**,
  **M-LOG-NOT-PRINT**).
- **Versions & lints** : source unique dans `[workspace.dependencies]`
  (**M-CARGO-WORKSPACE**), lints clippy all+pedantic au niveau workspace
  (**M-STATIC-VERIFICATION**), overrides via `#[expect]` (**M-LINT-OVERRIDE-EXPECT**).
- **Async** : `tokio` ; fonctions `async` plutôt que renvoyer des `Future`
  (**M-ASYNC-FN**).

## 7. Points durs {#points-durs}

1. **Consensus (Raft).** Réécrire un consensus production-grade est un projet en soi.
   Candidat : `openraft`. Intégration délicate (snapshot, membership dynamique,
   compatibilité on-disk du log). → Phase 2, précédée d'un spike.
2. **FEEL (langage d'expression).** Pas d'implémentation Rust mûre. À écrire (parser +
   évaluateur) ou binder, ou restreindre à un sous-ensemble. → Décision prise en début
   de Phase 3.
3. **Déterminisme du stream processor.** Le cœur repose sur un rejeu déterministe du
   log. Toute source de non-déterminisme (horloge, itération de hashmap, flottants,
   ordre d'E/S) doit être contrôlée. `testkit` fournit une horloge déterministe ;
   l'engine ne lit jamais le temps « réel » directement.
