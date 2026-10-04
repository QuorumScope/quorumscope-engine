#set document(title: "QuorumScope freeze episode report", author: "QuorumScope Engine")
#set page(paper: "a4", margin: (x: 2cm, y: 2cm))
#set text(size: 10.5pt)

#let data = json("data.json")

= QuorumScope freeze episode report

#table(
  columns: (auto, 1fr),
  stroke: none,
  [*Episode*], raw(data.incident_id),
  [*Network*], data.network,
  [*Basis*], data.basis,
  [*Status*], data.status,
  [*Opened at ledger*], str(data.opened_ledger),
  [*Closed at ledger*], if data.closed_ledger == none [Not closed] else [#data.closed_ledger],
  [*Generated from*], [QuorumScope database state at report time],
)

== Frozen keys changed during the episode

#if data.keys.len() == 0 [
  No key changes are recorded for this episode.
] else [
  #table(
    columns: (1fr, auto),
    [*Key hash*], [*Kind*],
    ..data.keys.map(k => (raw(k.key_id), if k.kind == none [not stored] else [#k.kind])).flatten()
  )
]

== Event timeline

#if data.events.len() == 0 [
  No events are recorded for this episode.
] else [
  #table(
    columns: (auto, auto, 1fr),
    [*Ledger*], [*Event*], [*Evidence*],
    ..data.events.map(e => (str(e.ledger), e.kind, raw(e.evidence_ref))).flatten()
  )
]

== Impact snapshots

#if data.snapshots.len() == 0 [
  No impact snapshots are recorded for this episode.
] else [
  #table(
    columns: (auto, auto, auto, auto),
    [*Ledger*], [*Frozen accounts*], [*Frozen trustlines*], [*Active bypasses*],
    ..data.snapshots.map(s => (str(s.ledger), str(s.frozen_accounts), str(s.frozen_trustlines), str(s.bypassed_transactions))).flatten()
  )
]

Counts come from the stored freeze set at each ledger. They describe frozen ledger keys, not application dependencies.
