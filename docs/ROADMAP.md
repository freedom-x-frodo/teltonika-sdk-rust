# tektonika-rs — Roadmap: Deferred Write-Path Features

Status: deferred from v0.1. The v0.1 SDK surface is **read-only** (link telemetry,
device health, data usage, QoS verification). This document plans the two write
capabilities that were consciously excluded, and the requirements they must meet
before implementation.

Rationale for deferral: both features mutate a router that is the robot's **only
WAN path** (5G behind CGNAT). A bad write doesn't degrade the system — it
disconnects it, with no inbound path for remote recovery. Write access therefore
needs safeguards that the current SDK (no re-auth, no request retry, no
confirmation model) cannot yet provide.

---

## 1. Recovery Actions (modem restart, device reboot)

### Purpose
Remotely recover a wedged modem or router without a field visit.

### Endpoints (verify per firmware before implementation)
- Modem/mobile connection restart (least destructive, first to implement)
- Full device reboot (last resort)

### Key risk
The action severs the link it is commanded through. The HTTP response may never
arrive even on success; loss of connectivity is the *expected* outcome, not an
error. Timeout/error handling semantics are inverted relative to every read call.

### Requirements before implementation
1. **Session re-auth must exist first.** Recovery is needed most during flaky
   connectivity, exactly when the 5-min session token has likely expired.
   Blocked by: token-expiry/re-auth work (`RwLock<AuthState>` + retry-on-401).
2. **Escalation order in API design.** Modem restart and device reboot are
   separate methods; docs must push callers to the least destructive option.
3. **Explicit-confirmation API shape.** Destructive calls must not be
   triggerable by accident, e.g. a required marker argument or a distinct
   `RecoveryApi` accessor — decide during design review.
4. **Fire-and-forget semantics.** Define a dedicated result type: "command
   accepted / link dropped as expected / genuine failure". Do not reuse
   `TeltonikaError::Network` for the expected disconnect.
5. **Post-action verification helper.** Poll device uptime after reconnect to
   confirm the reboot actually happened.
6. **Fleet-layer guardrails (out of SDK scope, document only):** rate-limit
   reboots per robot per hour; never auto-reboot while a delivery is active.

---

## 2. QoS / DSCP Configuration (write)

### Purpose
Provision the QoS policy that prioritizes the teleop control/heartbeat channel
above video (per system spec: deadman must win on a congested uplink).

### v0.1 stance
Read/verify only: the SDK checks that the expected QoS rules are present and
reports drift. Provisioning remains manual.

### Key risk
QoS rules are traffic-filtering config. A malformed rule can throttle or drop
the control channel itself — silently, and only under congestion, making the
failure hard to attribute. Config writes on RutOS also typically require an
apply/commit step; a partial write leaves the router in an inconsistent state.

### Requirements before implementation
1. **Read/verify path proven in production first.** The verify code defines the
   canonical rule set; write is "make it so" against the same model.
2. **Declarative, not imperative API.** Caller supplies a full desired QoS
   policy (typed struct, versioned); SDK diffs against device state and applies.
   No "add rule" / "delete rule" primitives — they invite drift.
3. **Transactional apply.** Investigate RutOS config apply/rollback semantics
   (uci-style commit, confirm-timeout if available). If the device supports
   apply-with-rollback-timer, use it: a write that breaks connectivity
   auto-reverts.
4. **Post-apply verification.** Re-read config and compare to intent; report
   drift as a typed error.
5. **Dry-run mode.** Diff-only call returning what *would* change, for fleet
   rollout tooling.
6. **Firmware-version gating.** QoS endpoint schema varies across RutOS
   versions; validate firmware before writing (revisits the deleted
   `DeviceType` validation idea — with a real consumer this time).

---
