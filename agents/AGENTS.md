# AGENTS.md — Glyph

CLI en Rust (git-like, sin UI) que compila un lenguaje intermedio compacto
(*glyph*) a Python real, con ofuscación por dialecto seed-derivado y respaldo
recuperable (frases multilingües + Shamir). Es un proyecto **independiente**
dentro de `appswebqueseyo`, pero se desarrolla por su cuenta.

Plan de fases (F0–F6) y su estado: ver [`ROADMAP.md`](./ROADMAP.md).

## Stack y comandos

- **Rust 2021**, crate `glyph` (lib + bin). Dependencias: `clap` (derive),
  `anyhow`, `sha2`, `blake3`, `rand`, `hkdf`, `aes-gcm`, `serde`, `toml`.
- **Compilar:** `cargo build` (release: `cargo build --release`, con `lto`).
- **Probar:** `cargo test` (lib: 14 tests; prop: 4 tests en `tests/prop.rs`).
- **Fuzzing:** objetivos en `fuzz/` (cargo-fuzz).
- **Lint/typecheck:** no hay clippy configurado; `cargo build` debe pasar sin
  warnings de peso. El proyecto compila limpio salvo un `unused_mut` conocido.

## Estructura

```
src/
  main.rs            # CLI raíz (clap): Init, Seed, Build, Obfuscate, Reveal
  lib.rs
  cli/
    init_cmd.rs      # args de `init` (--secret/--seed/--phrase)
    seed_cmd.rs      # args y dispatch de `seed *`
    code_cmd.rs      # Build/Obfuscate/Reveal (delegán en compiler)
  seed/
    prng.rs          # Seed ([u8;32]) + from_entropy/from_str
    crypto.rs        # encrypt_seed/encrypt_bytes (AES-GCM, clave HKDF(seed))
    dialect.rs       # dialect_id = blake3(seed)[:12]; parse_header; HEADER_PREFIX
    permute.rs       # build(seed) -> DialectMap (Fisher–Yates sobre CANONICAL_OPS)
    mnemonic.rs      # Glyph Mnemonic v1 (24 palabras, multi-idioma, checksum)
    shamir.rs        # split/combine GF(2^8) (polinomio AES 0x1b)
    wordlists.rs     # listas BIP-39 vendadas + fold/ detect_lang/ word_index
  store/
    init.rs          # almacén .glyph (init_project, load_seed, backup, recover...)
    layout.rs        # rutas .glyph/ , dialects/<id>/{seed,seed.enc}, config.toml
  compiler/
    ir.rs            # AST del IR (deriva PartialEq para round-trips)
    op.rs            # tablas binop/unop y helpers de forma (stmt/expr)
    glyph.rs         # S-expr glyfo: lexer, parser (glyph->IR) y emitter (IR->glyfo)
    py.rs            # front-end Python: lexer, parser (py->IR) y emitter (IR->py)
    mod.rs           # build / obfuscate / reveal (orquestación)
assets/wordlists/   # english/spanish/french/italian (+japanese/chinese_*) .txt + manifest.toml
```

## Invariantes clave

- **`dialect_id`** = `blake3(seed)[:12]`. Cabecera `#!glyph1 <id>\n`.
- **`CANONICAL_OPS`** es la lista canónica; `build()` aplica una permutación
  biyectiva (Fisher–Yates con PRNG derivado de la semilla) y asigna cada op a
  un glifo de `GLYPH_SOURCE`. Modificar `CANONICAL_OPS` cambia el dialecto de
  todos los proyectos existentes.
- **Ofuscación:** `obfuscate` = `parse_python -> IR -> emit_glyph -> AES-GCM`.
  `reveal` = inverso. `build` = `.glf` en claro (cabecera + texto glifo) -> py.
- **Glyph Mnemonic v1** NO es BIP-39 y NO es una cartera. 24 palabras =
  256 bits de entropía + 8 bits de checksum SHA-256 = 264 bits = 24×11 bits.
  Mismo seed -> frase equivalente en cada idioma registrado; cualquier frase
  completa recupera.
- **Shamir:** GF(2⁸) con polinomio del AES `0x1b`. `split(n,t)`; `combine`
  necesita ≥ `t` partes. Cada parte es `x || y` (33 bytes).
- **`.glyph/`** contiene la semilla (en claro o cifrada). Está en `.gitignore`.
  No subirlo sin criterio.

## Convenciones

- Sin comentarios en el código salvo que se pidan.
- El IR (`ir.rs`) deriva `PartialEq`: los round-trips `py -> IR -> glifo -> IR
  -> py` se prueban comparando la IR.
- Errores con `anyhow`; la CLI imprime `error: …` y sale con código 1.
- Mantén `cargo test` en verde al terminar.

## Limitaciones / advertencias de IA

- Es un proyecto en desarrollo; el formato glifo y el esquema de cabecera
  **no están congelados**.
- La criptografía (AES-GCM, HKDF) **no está auditada**. Funciona y las pruebas
  de round-trip pasan, pero no lo trates como criptografía de grado producción
  sin revisión externa.
- El subconjunto Python es limitado (ver README "Limitaciones conocidas").
  Al extender el front-end, actualiza `py.rs` (lexer+parser+emitter) y añade
  una prueba de round-trip en `py::tests`.
- Al tocar el lexer/parser de `glyph.rs` o `py.rs`, recuerda que la indentación
  de Python se maneja en el lexer (medición al inicio de cada línea lógica) y
  que `emit_expr` ya envuelve los operadores en `()`, por lo que `Stmt::Expr`
  NO debe volver a envolver.

## Cómo extender

- **Nuevo lenguaje objeto:** añade un emitter en `compiler/` que consuma `ir.rs`
  y un front-end opcional; `mod.rs` orquesta.
- **Nueva op:** añádela a `CANONICAL_OPS` (`seed/mod.rs`) y a las tablas de
  `op.rs`; el dialecto y el emitter de glifo la heredan.
- **Nuevo idioma de recuperación:** añade la lista en `wordlists.rs` (`LANGS`)
  y el `.txt` en `assets/wordlists/` con su entrada en `manifest.toml`.
