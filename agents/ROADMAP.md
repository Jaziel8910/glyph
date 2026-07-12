# ROADMAP.md — Glyph

Plan de fases del proyecto Glyph. Cada fase es independiente y verificable por
sí misma. Las fases **F0** y **F1** están hechas y verificadas; **F2–F6** están
planeadas (no iniciadas). Ver también `AGENTS.md` para estructura e invariantes.

Estado rápido:

| Fase | Qué es                                  | Estado      |
|------|-----------------------------------------|-------------|
| F0   | Motor de semilla, almacén y recuperación| ✅ Hecho    |
| F1   | Compilador núcleo (IR + glifo + Python) | ✅ Hecho    |
| F2   | Emitters multi-lenguaje                 | 📋 Planeado |
| F3   | Cobertura ampliada del fuente           | 📋 Planeado |
| F4   | Endurecido de formato glifo             | 📋 Planeado |
| F5   | i18n de recuperación (ja/zh)            | 📋 Planeado |
| F6   | Distribución y tooling                  | 📋 Planeado |

---

## F0 — Motor de semilla, almacén y recuperación ✅

Semilla de 256 bits (`[u8;32]`), dialecto derivado (`dialect_id = blake3(seed)[:12]`),
y recuperación sin dependencias externas.

- `init`: genera semilla + dialecto, escribe `.glyph/`.
- `seed show/new/rotate/verify/lock/unlock/export/import`.
- **Glyph Mnemonic v1**: frase de 24 palabras (BIP-39 style, NO es BIP-39, NO es
  cartera) en cualquier idioma registrado; mismo seed → frase equivalente en cada
  idioma; cualquier frase completa recupera.
- **Shamir**: `seed split` / `seed combine` (GF(2⁸), polinomio AES `0x1b`).
- **Backup/recover**: `seed backup --equivalent en,es,fr,it` y `seed recover
  --phrase`; `seed backup`/`recover --input` para blob cifrado.

Verificado por CLI y por 14 tests de lib + 4 property tests.

## F1 — Compilador núcleo ✅

IR (`ir.rs`) + representación glifo S-expr (`glyph.rs`) + front-end Python
(`py.rs`), orquestados en `mod.rs`.

- `build`: `.glf` en claro (cabecera + texto glifo) → Python.
- `obfuscate`: `.py` → `parse_python → IR → emit_glyph → AES-GCM` → `.glf`.
- `reveal`: `.glf` → descifra → glifo → IR → Python.

Subconjunto Python soportado: defs, asignación (incl. aug-assign), if/elif/else,
while, for, return, break/continue/pass, import/from-import (+as), literales,
binops con precedencia, `not`, llamadas, attr, subscript, list/tuple/dict,
ternario. **No** soportado: clases, decoradores, comprehensions, async, lambdas,
slicing, f-strings.

`obfuscate` → `reveal` hace round-trip idéntico (probado por CLI y por tests).

---

## F2 — Emitters multi-lenguaje 📋

Consumir la misma IR (`ir.rs`) para emitir código en más lenguajes objeto.

- Añadir `emit_js`/`emit_ts` (y luego `emit_rust`, etc.) en `compiler/`.
- `mod.rs` elige el emitter según extensión o flag (`--lang`).
- Cada emitter debe tener su prueba de round-trip `IR → código → IR` cuando el
  lenguaje objeto también tenga front-end (F3).

## F3 — Cobertura ampliada del fuente 📋

Ampliar el front-end Python y/o añadir nuevos front-ends que produzcan la IR.

- En `py.rs`: clases, `async`/`await`, comprehensions, lambdas, slicing, f-strings,
  gestores de contexto (`with`), `match` (3.10+).
- Opcional: front-ends para otros lenguajes fuente que parseen a la misma IR.
- Regla: cualquier extensión de `py.rs` (lexer+parser+emitter) lleva su prueba de
  round-trip en `py::tests` (ver advertencias de IA en `AGENTS.md`).

## F4 — Endurecido de formato glifo 📋

El formato glifo y el esquema de cabecera hoy **no están congelados**.

- Versionar y congelar el formato (semántica de `#!glyphN` y del cuerpo).
- En `build`: validar firma de dialecto (el `.glf` debe corresponder al dialecto
  del proyecto) y rechazar dialectos desconocidos.
- Migraciones de esquema de cabecera y detección de `.glf` corruptos.

## F5 — i18n de recuperación 📋

Habilitar `ja` / `zh-hans` / `zh-hant` en `wordlists.rs`.

- Normalización **NFKD** para japonés/chino (descomponer y plegar marcas).
- Manejo de espacios ideográficos / ancho completo en el parseo de frases.
- Las listas ya están vendadas en `assets/wordlists/`; solo falta registrarlas en
  `LANGS` y ajustar `fold`/`detect_lang`.

## F6 — Distribución y tooling 📋

Dejar el proyecto listo para usar y mantener.

- **Fuzz**: cablear y comprobar los objetivos en `fuzz/` (instalar `cargo-fuzz`).
- **CI**: GitHub Actions (build + `cargo test` en stable/beta).
- **Empaquetado**: `cargo install`, releases con binarios.
- **Editor/LSP** (opcional): resaltado o autocompletado del S-expr glifo.

---

## Tareas transversales pendientes

Independientes de las fases, pero necesarias para dejar el repo en estado limpio:

- Añadir `LICENSE` (MIT, según `LEGAL.md` / `AGENTS.md`).
- `git init` + commit inicial (el proyecto se movió fuera de `appswebqueseyo` y
  hoy no es repo; `.glyph/` y `glyph-shares/` ya están en `.gitignore`).
- Smoke test por CLI de `build` (`.glf` en claro → py) y de
  `backup` (blob cifrado) → `recover --input`.
- Configurar `clippy` y limpiar el `unused_mut` conocido.
