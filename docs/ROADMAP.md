# tektonika-rs — Roadmap

SDK for Teltonika RutOS devices (RUTX50). Consumed by the B2W teleop stack; the
router is the robot's **only WAN path** (5G behind CGNAT).

API contract is pinned to the vendored spec: `specs/RUTX50_7.23.7_v1.15.1.json`
(Vuci API 1.15.1). All paths below are relative to `https://<host>/api`.

**v0.1 scope: read-only.** Link telemetry, device health, data usage, QoS
verification. Write paths are deferred — see §4.

---

## 1. Implementation sequence

Spec-driven, in order. Each item is verified against the vendored spec before
coding.

1. ~~`LoginResponse` → verified shape (`data.token`)~~ — done
2. ~~**`api/` namespace skeleton.** Accessor pattern — `client.system()`,~~
   ~~`client.mobile()` returning borrowing structs; `RestClient` keeps only~~
   ~~transport (auth, verb helpers, envelope, error mapping). Migrate~~
   ~~`device_status` out of `client.rs`. Prevents the flat-impl bloat that a~~
   ~~976-endpoint API guarantees.~~
3. **`modems` namespace.**
   ~~- `GET /modems/status` — array, two schema variants (full / offline stub).~~
     ~~Type ~15 of 84 fields: `rsrp`, `rsrq`, `sinr`, `rssi`, `ntype`, `operator`,~~
     ~~`state`, `conntype`, `simstate`, `active_sim`, `txbytes`, `rxbytes`,~~
     ~~`temperature`, `id`, `model`. All `Option` — covers both variants.~~
   - `GET /internet_connection/status` — `{dns_status, ipv4_status, ipv6_status}`;
     reachability, distinct from radio registration.
   - `GET /failover/status` — `{interface_name}`; active WAN.
4. **Data usage.** Two overlapping families in the spec:
   `/data_usage/{interval}/...` (per-modem, per-SIM) vs
   `/network_usage/metrics/{day|week|month|total}/status`. Decide after dumping
   both from a live device. Per-SIM variant likely fits fleet cost tracking.
5. **QoS read/verify.** `GET /qos/interfaces/config`, `GET /qos/rules/config`,
   `GET /qos/rules/options`. Rule item schema is dynamic (uci-backed) and does
   not resolve statically in the spec — **requires a live device dump before
   typing.** Verify-only: report drift from the expected policy, do not write.

## 2. Blocking / parallel work

- **Token re-auth.** Session token expires after 5 minutes.
  `RwLock<AuthState>` + retry-once-on-401, storing credentials for re-login;
  `GET /session/status` (`{active, username, group}`) as the probe. Also
  replaces the stubbed 401 branch in `get()`, which currently returns
  `AuthFailed` with an empty username — wrong story for an expiry.
  **Prerequisite for everything in §4.**
- ~~**Vendor the spec** into `specs/` so firmware bumps become reviewable diffs.~~

## 3. Deferred: write-path features

Rationale for deferral: both mutate the robot's only WAN path. A bad write
doesn't degrade the system — it disconnects it, with no inbound path for remote
recovery. Write access needs safeguards the current SDK (no re-auth, no request
retry, no confirmation model) cannot yet provide.

### 3.1 Recovery actions

**Purpose.** Remotely recover a wedged modem or router without a field visit.

**Endpoints (confirmed present in the vendored spec).** Three-step escalation
ladder, least destructive first:

1. `POST /modems/{id}/actions/restart_connection` — re-establish the data
   connection only.
2. `POST /modems/{id}/actions/reboot` — modem power-cycle.
3. `POST /system/actions/reboot` — full device reboot, last resort.

Both modem actions take an `id` from `GET /modems/status` — the recovery API
cannot be a bare no-arg call.

**Key risk.** The action severs the link it is commanded through. The HTTP
response may never arrive even on success; loss of connectivity is the
*expected* outcome, not an error. Timeout and error semantics are inverted
relative to every read call.

**Requirements before implementation.**

1. **Session re-auth must exist first** (§2). Recovery is needed most during
   flaky connectivity — exactly when the 5-minute token has expired.
2. **Escalation order encoded in the API.** Three separate methods; naming and
   docs must push callers to the least destructive option that fits.
3. **Explicit-confirmation API shape.** Destructive calls must not be
   triggerable by accident — a required marker argument or a distinct
   `RecoveryApi` accessor. Decide at design review.
4. **Fire-and-forget semantics.** Dedicated result type distinguishing
   "command accepted / link dropped as expected / genuine failure". Do not
   reuse `TeltonikaError::Network` for the expected disconnect.
5. **Post-action verification helper.** Poll device uptime after reconnect to
   confirm the reboot actually happened.
6. **Fleet-layer guardrails (out of SDK scope, document only).** Rate-limit
   reboots per robot per hour; never auto-reboot during an active delivery.

### 3.2 QoS / DSCP configuration (write)

**Purpose.** Provision the policy that prioritizes the teleop control/heartbeat
channel above video — per system spec, the deadman command must win on a
congested uplink.

**v0.1 stance.** Read/verify only (§1.6). Provisioning stays manual.

**Key risk.** QoS rules are traffic-filtering config. A malformed rule can
throttle or drop the control channel itself — silently, and only under
congestion, making the failure hard to attribute. RutOS config writes also
require an apply/commit step; a partial write leaves an inconsistent state.

**Requirements before implementation.**

1. **Read/verify path proven in production first.** The verify code defines the
   canonical rule set; write is "make it so" against the same model.
2. **Declarative, not imperative API.** Caller supplies a full desired policy
   (typed, versioned); SDK diffs against device state and applies. No
   add-rule / delete-rule primitives — they invite drift.
3. **Transactional apply.** Investigate RutOS apply/rollback semantics
   (uci-style commit, confirm-timeout). If apply-with-rollback-timer exists,
   use it: a write that breaks connectivity auto-reverts.
4. **Post-apply verification.** Re-read config, compare to intent, report drift
   as a typed error.
5. **Dry-run mode.** Diff-only call returning what *would* change, for fleet
   rollout tooling.
6. **Firmware-version gating.** QoS schema varies across RutOS versions;
   validate firmware before writing. Revisits the deleted `DeviceType`
   validation idea — with a real consumer this time.