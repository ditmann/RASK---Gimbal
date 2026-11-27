# RASK-Gimbal

RASK-Gimbal er et prosjekt for å styre en tre-akset kameragimbal med bevegelsene til et VR-headset. Systemet bruker tre steppermotorer for rotasjon rundt **yaw**, **pitch** og **roll**, og er utviklet i Rust med en klient–server-arkitektur.

Klienten kjøres på en Windows-PC og leser orienteringen til VR-headsettet gjennom **SteamVR/OpenVR**. Orienteringen gjøres om til ønskede motorposisjoner og sendes som UDP-pakker til en **Raspberry Pi**, som styrer motorene med STEP- og DIR-signaler.

Prosjektet er utviklet i forbindelse med en robotikkoppgave for Institutt for energiteknikk (IFE).

## Hvordan systemet fungerer

```text
VR-headset
    |
    v
Windows-PC (Client)
    |  SteamVR / OpenVR
    |  Rotasjonsmatrise -> yaw, pitch, roll -> steps
    |  UDP (port 5000)
    v
Raspberry Pi (Server)
    |  Mottak og validering av pakker
    |  Motorstyring med hastighets- og akselerasjonsbegrensninger
    v
STEP/DIR-motordrivere
    |
    v
Tre steppermotorer (yaw, pitch, roll)
```

Nettverkskommunikasjonen og motorstyringen kjører i separate tråder på serveren. Nye målposisjoner prioriteres fremfor eldre, og serveren stopper pulsgenereringen dersom det ikke kommer nye kommandoer innen **200 ms**.

**Avgrensning:** Denne kodebasen håndterer orienteringsdata og motorstyring. Overføring av kameravideo til VR-headsettet er ikke implementert her.

## Forutsetninger

- **Rust og Cargo** (prosjektet bruker Rust 2024-edisjonen).
- **Windows-PC** med SteamVR og et kompatibelt VR-headset for kjøring av VR-klienten.
- **Raspberry Pi** med Linux (ARM/AArch64), steppermotorer og eksterne STEP/DIR-motordrivere for fysisk motorstyring.
- Nettverk mellom PC-en og Raspberry Pi-en, med tilgang til **UDP-port 5000**.

For bygging av VR-klienten på Windows kan `openvr` / `openvr_sys` kreve C++-byggeverktøy, LLVM/Clang og `libclang`. Prosjektet har noen oppsettnotater i [`docs/requiremnts.txt`](docs/requiremnts.txt). Stiene der er maskinspesifikke og må eventuelt tilpasses din egen PC.

## Kom i gang

Kjør Cargo-kommandoene fra prosjektets rotmappe, der `Cargo.toml` ligger.

### 1. Angi serverens IP-adresse

Klientprogrammene bruker foreløpig en **hardkodet IP-adresse**. Endre adressen til Raspberry Pi-en i disse filene før du kjører dem:

| Program | Fil | Verdi som må endres |
| --- | --- | --- |
| VR-klient | `Client/src/ClientAPI.rs` | `destination` i `main()` |
| Manuell klient | `Client/src/manual_gimbal.rs` | `SERVER_ADDRESS` |
| Testklient | `Client/src/mock_gimbal.rs` | `SERVER_ADDRESS` |

I den nåværende koden brukes `10.219.103.53:5000`. Endre dette til IP-adressen til din Raspberry Pi, med port `5000`. Serveren lytter på `0.0.0.0:5000`.

### 2. Start serveren på Raspberry Pi

Kjør på Raspberry Pi-en:

```bash
cargo run --release --bin server
```

Serveren lytter etter UDP-pakker og sender STEP- og DIR-signaler til motordriverne. Kontroller GPIO-koblingene og innstillingene på driverne **før motorene får strøm**.

Du kan også kjøre serverprogrammet på Windows for å teste nettverks- og motorstyringslogikken. Da brukes **simulerte motordrivere** i stedet for GPIO, og ingen fysiske motorer styres:

```bash
cargo run --bin server
```

### 3. Start en klient

Velg én av følgende klienter på PC-en.

**VR-klient** – leser orientering fra SteamVR/OpenVR:

```bash
cargo run --release --bin client --features vr
```

Start SteamVR og koble til VR-headsettet først. Programmet venter dersom SteamVR eller headsettet ikke er tilgjengelig.

**Manuell klient** – send målposisjoner fra terminalen, uten VR-utstyr:

```bash
cargo run --bin manual
```

Eksempler på kommandoer i den manuelle klienten:

```text
yaw 20
pitch -10
roll 5
set yaw 40
zero
status
help
quit
```

`yaw 20` flytter målet 20 steps fra nåværende mål, mens `set yaw 40` setter en absolutt målposisjon. Den manuelle klienten begrenser målene til ±150 steps for yaw og ±40 steps for pitch/roll. Disse grensene gjelder **den manuelle klienten**, ikke nødvendigvis andre klienter eller serveren.

**Mock-klient** – sender sinusformede målposisjoner uten VR-utstyr:

```bash
cargo run --bin mock
```

Mock-klienten er nyttig for å teste retningsendringer og gjentatte bevegelser. Stopp den med `Ctrl+C`.

## Kommunikasjonsformat

Klienten sender én UDP-pakke på **20 byte**. Alle tall overføres i **little-endian**-format.

| Byte | Innhold | Rust-type |
| --- | --- | --- |
| 0–7 | Tidsstempel i millisekunder | `u64` |
| 8–11 | Ønsket yaw-posisjon i steps | `i32` |
| 12–15 | Ønsket pitch-posisjon i steps | `i32` |
| 16–19 | Ønsket roll-posisjon i steps | `i32` |

Tidsstempelet brukes til å forkaste eldre pakker. VR-klienten har en dødsone på **2 steps** for å redusere små bevegelser fra sensorstøy. Den sender nye mål når de endres, eller en keepalive-pakke minst hver **100 ms** så lenge den mottar gyldige orienteringsdata.

## Motorstyring og GPIO

Motorene styres med STEP- og DIR-signaler gjennom `rppal` på Raspberry Pi. Følgende GPIO-pinner er definert i koden (**BCM-nummerering**):

| Akse | STEP | DIR | Maksimal pulsfrekvens |
| --- | --- | --- | --- |
| Yaw | GPIO 17 | GPIO 27 | 1000 pulser/s |
| Pitch | GPIO 24 | GPIO 25 | 300 pulser/s |
| Roll | GPIO 22 | GPIO 23 | 300 pulser/s |

Motorstyringen bruker separate hastighetsprofiler for hver akse (`MotorRamp`). Beregningene tar høyde for akselerasjon, bremselengde og retningsendringer. Programmet er satt opp for **400 steps per motoromdreining**, og klienten bruker **3:1 girutveksling på yaw**. Innstillingene for mikrostepping på de fysiske driverne må stemme med dette.

> **Merk:** Systemet er åpent styrt (uten posisjonsfeedback fra motorene). Programmet holder styr på antall utsendte steps, men kan ikke selv oppdage tapte motorsteg. Kalibrering og forsvarlig testing er derfor viktig. Linux gir heller ingen garanti for helt presis timing av STEP-pulsene.

## Testing

Prosjektet inneholder **31 automatiserte tester**: 10 for klienten og 21 for serveren. De dekker blant annet omregning fra rotasjonsmatriser til steps, UDP-pakker, dødsone og keepalive, målposisjoner, `MotorRamp` og håndtering av stopp/timeout.

Kjør testene uten VR-avhengigheten:

```bash
cargo test --no-default-features
```

Testene for motorstyring bruker simulerte motordrivere på Windows og er ikke en erstatning for testing av faktiske motorer, ledninger eller GPIO-timing.

GitHub Actions-konfigurasjonen i [`.github/workflows/rust-tests.yml`](.github/workflows/rust-tests.yml) er satt opp til å kjøre den samme testkommandoen på Windows ved push, pull request og manuell start.

## Prosjektstruktur

```text
RASK-Gimbal/
├── .github/workflows/rust-tests.yml   # Automatiske tester
├── Cargo.toml                         # Rust-pakke og kjørbare programmer
├── Client/src/
│   ├── ClientAPI.rs                   # VR-klient, UDP og filtrering
│   ├── RotationMath.rs                # Rotasjonsmatrise -> vinkler -> steps
│   ├── vr.rs                          # SteamVR/OpenVR-integrasjon
│   ├── manual_gimbal.rs               # Manuell styring
│   └── mock_gimbal.rs                 # Simulert klientinput
├── Server/src/
│   ├── main.rs                        # Oppstart av serveren
│   ├── server_api.rs                  # Mottak av UDP-pakker
│   ├── packet.rs                      # Dekoding av pakker
│   ├── gimbal_controller.rs           # Styring av de tre motorene
│   ├── motor_loop.rs                  # MotorRamp, pulstiming og timeout
│   └── motor_driver/                  # GPIO-drivere / simulator
├── tests/
│   ├── client/                        # Klienttester
│   └── server/                        # Servertester
└── docs/                              # Oppsettnotater og arkitekturillustrasjon
```

## Kjente begrensninger

- IP-adressene til serveren må endres i kildekoden ved bytte av nettverk.
- Videooverføring fra kamera til VR-headset er ikke en del av denne implementasjonen.
- Motorposisjonen beregnes ut fra utsendte steps, uten tilbakemelding fra motorene.
- Timing av GPIO-pulser avhenger av Linux og kan variere under belastning.
- Det er nødvendig å kontrollere strømforsyning, kabling, mekaniske begrensninger og kalibrering før fysisk bruk.
