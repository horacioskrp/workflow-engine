# Documentation

Documentation d'ingénierie du moteur de workflow (Rust).

| Document | Contenu |
|---|---|
| [architecture.md](architecture.md) | **Constitution** : vision, capacités & supports, les 15 crates, graphe de dépendances, modèle d'exécution runtime, décisions structurantes, points durs. |
| [roadmap.md](roadmap.md) | **Toutes les étapes d'implémentation**, phase par phase (0 → 5) : objectifs, crates, étapes concrètes, livrables (DoD), dépendances externes, risques. |

## État actuel

**Phase 0 — en cours.** Le workspace Cargo (16 crates), les lints, le contrat gRPC
(`proto/gateway.proto`) câblé via tonic-build (`contracts::v1`), un démon `node` qui
démarre, la **CI** (fmt + clippy + test) et le flux **gitflow** sont en place. Les
crates métier sont encore des stubs documentés. Le projet compile et s'exécute via
Docker (voir le [README racine](../README.md)). Suivi détaillé dans
[roadmap.md](roadmap.md).

Processus de contribution (gitflow + CI) : voir [CONTRIBUTING](../CONTRIBUTING.md).

## Principe d'organisation

Découpage **par capacité métier** (et non par couches d'infrastructure) :

- **Capacités** : `workflow` (exécution), `scheduling` (timers), `tasks` (work items),
  `messaging` (signaux/corrélation).
- **Supports** : `persistence` (état+historique), `coordination` (cluster/réplication),
  `api` (gRPC), `feed` (flux sortant), `expr` (expressions).
- **Socle/bordure** : `kernel`, `contracts`, `node` (démon), `ctl` (CLI), `sdk`,
  `harness`.

Vocabulaire et découpage **propriétaires** : conçus pour ce projet, indépendamment de
tout moteur existant.

## Rappels de cadrage

- **Compatibilité visée : API-only.** On reproduit un contrat client gRPC ; les formats
  internes (historique, snapshots, protocole réseau) sont les nôtres.
- **Conventions** : Pragmatic Rust Guidelines (ids `M-*`). Édition 2024, `unsafe`
  interdit par défaut, erreurs canoniques en structs, `anyhow` dans les binaires.
- Trois **points durs** : coordination/réplication, langage d'expression (`expr`), et
  **déterminisme** de `workflow`. Détaillés dans
  [architecture.md](architecture.md#points-durs).
