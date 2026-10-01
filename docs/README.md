# Documentation

Documentation d'ingénierie du moteur de workflow (Rust).

| Document | Contenu |
|---|---|
| [architecture.md](architecture.md) | **Constitution** : vision, capacités & supports, les 15 crates, graphe de dépendances, modèle d'exécution runtime, décisions structurantes, points durs. |
| [roadmap.md](roadmap.md) | **Toutes les étapes d'implémentation**, phase par phase (0 → 5) : objectifs, crates, étapes concrètes, livrables (DoD), dépendances externes, risques. |

## État actuel

**Phase 0 — scaffold livré.** Le workspace Cargo (16 crates), le câblage, les lints,
le contrat gRPC (`proto/gateway.proto`) et un démon `node` qui démarre sont en place.
Les crates sont des stubs documentés ; aucune logique métier encore. Le projet compile
et s'exécute via Docker (voir le [README racine](../README.md)).

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
