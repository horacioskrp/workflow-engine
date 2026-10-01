# Architecture — constitution & capacités

## 1. Vision

Une plateforme d'orchestration de processus métier : les clients déploient des
définitions de processus et pilotent des instances via une **API** ; la plateforme
exécute ces processus de façon fiable, distribue le travail à des workers, réagit au
temps et aux messages, et conserve un historique durable.

Principe d'organisation : **par capacité métier**. Chaque crate correspond à *ce que
la plateforme fait*, pas à une couche technique. Principe de conception :
**M-RUST-SHAPED** — patterns Rust, types concrets, peu d'indirection
(**M-SIMPLE-ABSTRACTIONS**, **M-DI-HIERARCHY**).

## 2. Les capacités et leurs supports

```
┌──────────────────────── CAPACITÉS MÉTIER ────────────────────────┐
│  workflow     scheduling     tasks        messaging               │
│  (exécution)  (timers)       (work items) (signaux/corrélation)   │
└───────────────────────────────────────────────────────────────────┘
┌──────────────────────── SUPPORTS ────────────────────────────────┐
│  persistence   coordination   api      feed        expr           │
│  (état+histo)  (cluster/répl) (gRPC)   (flux sortant)(expressions) │
└───────────────────────────────────────────────────────────────────┘
┌──────────────────────── SOCLE / BORDURE ─────────────────────────┐
│  kernel (primitives)   contracts (schéma API)                     │
│  node (démon)   ctl (CLI)   sdk (client)   harness (tests)        │
└───────────────────────────────────────────────────────────────────┘
```

## 3. Les crates

| Crate | Capacité / rôle | Techno clé | Phase |
|---|---|---|:--:|
| `kernel` | Primitives : identifiants, horloge, télémétrie, scaffolding d'erreur | std | 0 |
| `contracts` | Types du schéma d'API générés depuis `proto/gateway.proto` | tonic-build, prost | 0 |
| `expr` | Langage d'expression : parser + évaluateur | — (à écrire) | 3 |
| `persistence` | État durable + historique + snapshots | rocksdb, serde | 1 |
| `scheduling` | Timers, échéances, backoff de retry | — | 3 |
| `tasks` | Work items + distribution aux workers | — | 3 |
| `messaging` | Signaux/messages + corrélation aux instances | — | 3 |
| `workflow` | **Cœur** : modèle de processus + exécution déterministe | serde | 3 |
| `coordination` | Mise en cluster + réplication de l'état persisté | openraft, rustls | 2 |
| `feed` | Flux sortant de l'historique committé vers l'extérieur | — | 4 |
| `api` | Interface gRPC externe, point d'entrée unique | tonic, tokio | 4 |
| `sdk` | SDK client (applications & workers) | tonic | 4 |
| `node` | **Démon** : assemble toutes les capacités | tokio, anyhow, mimalloc | 0→ |
| `ctl` | CLI opérateur | anyhow | 4 |
| `harness` | Utilitaires de test : horloge déterministe, fakes | — | 0→ |

(+ `xtask` : automatisation de build/codegen, hors graphe applicatif.)

## 4. Graphe de dépendances (sens unique, sans cycle)

```
kernel ← (rien)
contracts ← (rien, généré)
expr          ← kernel
persistence   ← kernel
scheduling    ← kernel, persistence
tasks         ← kernel, persistence
messaging     ← kernel, persistence
workflow      ← kernel, persistence, expr, scheduling, tasks, messaging
coordination  ← kernel, persistence
feed          ← kernel, persistence
api           ← kernel, contracts, workflow, tasks
sdk           ← kernel, contracts
node          ← kernel, persistence, coordination, scheduling, tasks,
                messaging, workflow, feed, expr, api
ctl           ← kernel, sdk, contracts
harness       ← kernel
```

Les identifiants et types transverses (ex. `InstanceId`, `TaskId`, horloge) vivent
dans `kernel`, ce qui permet à `scheduling`/`tasks`/`messaging` d'être indépendants
les uns des autres ; `workflow` est le seul à les orchestrer. Respecte
**M-CRATES-FLAT-FOLDER** et **M-CRATES-IN-WORKSPACE**.

## 5. Modèle d'exécution runtime (comment ça s'assemble)

Le `node` est un membre du cluster. À l'exécution :

1. **Appartenance & ownership.** `coordination` établit l'appartenance au cluster et
   attribue à chaque node la responsabilité de tranches de charge, répliquées sur
   plusieurs nodes (l'une est primaire).
2. **Exécution.** Pour chaque tranche dont ce node est responsable, `workflow` fait
   avancer les instances de processus **de façon déterministe** : il lit l'historique
   via `persistence`, applique la sémantique d'exécution, puis écrit le nouvel état +
   les nouveaux événements. Au fil de l'exécution il sollicite :
   - `tasks` pour créer/distribuer le travail aux workers,
   - `scheduling` pour armer timers et échéances,
   - `messaging` pour s'abonner/corréler des signaux,
   - `expr` pour évaluer conditions et expressions de variables.
3. **Durabilité & réplication.** `persistence` conserve état + historique et produit
   des snapshots ; `coordination` réplique le tout pour la tolérance aux pannes.
4. **Entrée client.** `api` (gRPC) reçoit les appels, les valide, et pilote `workflow`
   et `tasks` sur le node responsable.
5. **Sortie.** `feed` suit l'historique committé et le pousse vers des systèmes
   externes (monitoring, analytique).

> Le **déterminisme** de l'étape 2 est la propriété centrale : rejouer le même
> historique doit reproduire exactement le même état sur chaque réplica et après
> redémarrage.

## 6. Décisions structurantes

- **Compatibilité API-only.** On fige le contrat `proto/gateway.proto` ; les formats
  internes (historique, snapshot, protocole inter-nodes) sont les nôtres.
- **Erreurs** : `Error` struct canonique avec `Backtrace` par crate biblio
  (**M-ERRORS-CANONICAL-STRUCTS**) ; `anyhow` dans les binaires (**M-APP-ERROR**) ;
  conversions via `From` (**M-FROM-ERROR**).
- **Sûreté** : `unsafe` interdit au niveau workspace (**M-UNSAFE**).
- **Observabilité** : `tracing` partout, logs structurés (**M-LOG-STRUCTURED**,
  **M-LOG-NOT-PRINT**).
- **Versions & lints** : source unique dans `[workspace.dependencies]`
  (**M-CARGO-WORKSPACE**) ; clippy all+pedantic workspace (**M-STATIC-VERIFICATION**) ;
  overrides via `#[expect]` (**M-LINT-OVERRIDE-EXPECT**).
- **Async** : `tokio` ; fonctions `async` plutôt que renvoyer des `Future`
  (**M-ASYNC-FN**).

## 7. Points durs {#points-durs}

1. **Coordination/réplication** — un consensus production-grade est un projet en soi
   (candidat `openraft`, dans `coordination`). → Phase 2, précédée d'un spike.
2. **`expr`** — pas d'implémentation Rust mûre d'un langage d'expression type FEEL ;
   à écrire, binder, ou restreindre. → Décision en début de Phase 3.
3. **Déterminisme** de `workflow` — contrôler toute source de non-déterminisme
   (horloge via `harness`, pas d'itération non ordonnée ni de flottants dans la
   logique de décision).
