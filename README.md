# Glyph

![Rust](https://img.shields.io/badge/rust-1.74%2B-orange)
![Estado](https://img.shields.io/badge/estado-en%20desarrollo-yellow)
![Licencia](https://img.shields.io/badge/licencia-ver%20LEGAL.md-blue)
![Cifrado](https://img.shields.io/badge/cifrado-AES--256--GCM-success)

**Glyph** es una interfaz de línea de comandos (CLI) pensada para que tanto
**agentes de IA** como **humanos** compilen un lenguaje intermedio compacto
(*glyph*) a código real, con ofuscación por dialecto derivada de una semilla
por proyecto, y un sistema de respaldo/recuperación multilingüe (frases tipo
BIP-39 + Shamir).

> Una semilla por proyecto genera un **dialecto**: el mismo código fuente se
> ofusca de forma distinta para cada proyecto, y solo quien tenga la semilla
> puede revelarlo. Sin la semilla, el `.glf` es ilegible.

---

## ¿Qué es Glyph?

Glyph tiene tres capas que conviven en un solo binario:

1. **Motor de semilla y almacén (`F0`)** — genera una semilla de 256 bits por
   proyecto, la protege (en claro o cifrada con passphrase) y ofrece frases de
   recuperación en varios idiomas y fragmentos de Shamir.
2. **Dialecto ofuscador** — a partir de la semilla se deriva un *dialect id*
   (`blake3`) y una permutación biyectiva de los operadores sobre un alfabeto
   de glifos Unicode. El mismo programa se escribe de forma distinta en cada
   proyecto.
3. **Compilador (`F1`)** — un subconjunto de Python se parsea a un árbol
   sintáctico intermedio (IR), se emite como *glyph* (S-expressions con
   operadores-glifo) y se cifra con AES-256-GCM. `reveal` invierte el proceso
   y `build` compila un `.glf` en claro a Python.

Glyph **no es un lenguaje de programación nuevo**: es un *transpilador* de un
subconjunto de Python, más una capa de ofuscación y de respaldo criptográfico.

---

## Características

- **Ofuscación por dialecto seed-derivado.** Cada proyecto tiene su propio
  alfabeto de glifos; revelar un `.glf` requiere la semilla exacta.
- **Cifrado real.** El cuerpo del `.glf` se cifra con AES-256-GCM (clave
  derivada por HKDF-SHA256 desde la semilla). Sin la semilla, no hay forma de
  leerlo.
- **Recuperación multilingüe.** La misma semilla se codifica como una frase de
  24 palabras en `en`, `es`, `fr` e `it`. Cualquier frase completa recupera la
  semilla.
- **Shamir secret sharing.** Divide la semilla en *N* partes; cualquier *T*
  reconstruye (GF(2⁸), polinomio del AES `0x1b`).
- **Transpilador Python → IR → glyph → Python** de ida y vuelta (round-trip
  idéntico en las pruebas).
- **Sin dependencias de red ni telemetría.** 100% local.

---

## Instalación

Necesitas la toolchain estable de Rust (1.74+):

```bash
# Desde el código fuente
cargo build --release
# El binario queda en target/release/glyph

# O instalar en tu cargo bin
cargo install --path .
```

No se publican binarios prefabricados.

---

## Inicio rápido

```bash
# 1) Inicializa un proyecto glyph en el directorio actual.
#    --secret guarda la semilla cifrada (pide passphrase al usarla).
glyph init --secret

# 2) Ofusca un .py real en un .glf cifrado.
glyph obfuscate ejemplo.py        # -> ejemplo.glf

# 3) Revela el .glf de vuelta a Python (pide la passphrase).
glyph reveal ejemplo.glf          # -> ejemplo.py (idéntico al original)

# 4) O compila un .glf en claro a Python.
glyph build fuente.glf            # -> fuente.py
```

### Ejemplo

```python
# ejemplo.py
def add(a, b):
    return a + b

x = add(2, 3)
if x > 0:
    print(x)
else:
    print(0)
```

```bash
glyph obfuscate ejemplo.py
# escribe ejemplo.glf:
#   #!glyph1 fc80080a3df7
#   <blob cifrado con AES-256-GCM, ilegible sin la semilla>

glyph reveal ejemplo.glf
# reconstruye ejemplo.py idéntico al original
```

---

## Cómo funciona

### Dialecto derivado de la semilla

```
seed (256 bits) ──blake3──▶ dialect_id   (12 caracteres, ej. "fc80080a3df7")
seed (256 bits) ──Fisher–Yates──▶ op ──▶ glifo  (permutación biyectiva)
```

La permutación se calcula sobre la lista canónica de operadores
(`CANONICAL_OPS`) y se asigna a los caracteres del alfabeto de glifos
(`GLYPH_SOURCE`, símbolos Unicode). El mismo operador (`add`, `if`, `call`, …)
se representa con un glifo distinto en cada proyecto.

### Tubería de ofuscación

```
 .py ──parse_python──▶ IR ──emit_glyph──▶ texto glifo ──AES-256-GCM(seed)──▶ .glf
 .glf ──AES-256-GCM⁻¹──▶ texto glifo ──parse_glyph──▶ IR ──emit_python──▶ .py
```

El `.glf` lleva en su primera línea una cabecera con el *dialect id*:

```
#!glyph1 fc80080a3df7
<bytes cifrados>
```

`build` (para `.glf` en claro) y `reveal` (para `.glf` cifrado) leen ese id,
cargan la semilla correspondiente del almacén y aplican la permutación inversa.

### Recuperación

```
seed (256 bits) ──SHA-256(checksum 8 bits)──▶ 24 palabras (estilo BIP-39)
                                      ├─ en
                                      ├─ es
                                      ├─ fr
                                      └─ it     (frases equivalentes)
```

Cualquier frase completa, en cualquiera de los idiomas registrados, recupera
la semilla. Véase **LEGAL.md**: el formato *Glyph Mnemonic* **no es BIP-39**
y **no es una cartera de criptomonedas**.

---

## Referencia de comandos

### `glyph init`

| Opción        | Descripción                                                |
| ------------- | ---------------------------------------------------------- |
| `--secret`    | Guarda la semilla cifrada (pide passphrase en cada uso).   |
| `--seed <s>`  | Inicializa desde una semilla textual en lugar de aleatoria.|
| `--phrase`    | Imprime la frase de 24 palabras tras inicializar.          |

### `glyph seed`

| Subcomando     | Descripción                                                        |
| -------------- | ------------------------------------------------------------------ |
| `show [--public]` | Muestra el *dialect id* y el mapa op→glifo (`--public`: solo id).|
| `new`          | Regenera la semilla del dialecto actual.                          |
| `export`       | Exporta el blob de la semilla a `glyph-seed-export.bin`.          |
| `import <f>`   | Importa una semilla desde un blob exportado.                      |
| `rotate`       | Crea un dialecto nuevo y lo hace actual (semillas previas se conservan).|
| `verify`       | Comprueba que la semilla coincide con el *dialect id* almacenado. |
| `lock`         | Cifra una semilla en claro con una passphrase.                    |
| `unlock`       | Descifra la semilla (pide passphrase).                            |
| `backup [--equivalent L] [--out F] [--pass P]` | Escribe un backup cifrado, o imprime frases equivalentes por idioma (`--equivalent en,es,fr,it`).|
| `recover [--input F] [--pass P] [--phrase F]` | Restaura desde un backup cifrado o desde una frase.          |
| `phrase [--lang L]` | Imprime la frase de recuperación en el idioma dado (def. `en`).|
| `split [--shares N] [--threshold T] [--lang L] [--out D]` | Divide la semilla en *N* partes Shamir (umbral *T*).|
| `combine <f...>` | Recombina *T* partes Shamir en la semilla.                     |

### `glyph build` / `obfuscate` / `reveal`

| Comando       | Entrada | Salida | Descripción                              |
| ------------- | ------- | ------ | ---------------------------------------- |
| `build <f>`   | `.glf`  | `.py`  | Compila un `.glf` en claro a Python.     |
| `obfuscate <f>` | `.py` | `.glf` | Python → IR → glifo → AES → `.glf`.      |
| `reveal <f>`  | `.glf`  | `.py`  | AES → glifo → IR → Python (inverso).     |

Todos aceptan `-o/--output` para fijar la ruta de salida.

---

## Modelo de seguridad

- **Semilla = raíz de confianza.** Todo (dialecto y cifrado) se deriva de ella.
  Quien tenga la semilla controla el proyecto.
- **Modo secreto.** Con `init --secret`, la semilla se cifra con AES-GCM usando
  una passphrase; el `.glyph/` resultante puede compartirse sin revelar la
  semilla en claro.
- **Ofuscación ≠ seguridad fuerte.** El objetivo de los glifos es *opacar* el
  código para que no sea legible de inmediato y para que cada proyecto use un
  dialecto distinto. No es ofuscación de nivel comercial ni protección contra
  un atacante determinado con la semilla.
- **El `.glf` cifrado sí es confidencialidad real** (AES-256-GCM) mientras la
  semilla permanezca secreta.

### Limitaciones conocidas

- El subconjunto de Python soportado es limitado: defs, asignación (incl.
  aumentada), `if/elif/else`, `while`, `for`, `return`, `break/continue/pass`,
  `import`/`from … import`, literales, operadores binarios/unarios con
  precedencia, llamadas, atributos, subíndices, listas/tuplas/diccionarios y
  ternarios. **No** soporta clases, decoradores, comprehensions, `async`,
  lambdas, slicing ni f-strings.
- El formato de glifo y el esquema de cabecera **no están congelados**; pueden
  cambiar entre versiones.
- `ja`/`zh-hans`/`zh-hant` están vendados pero **deshabilitados** en esta
  versión (requieren normalización NFKD / manejo de espacios ideográficos).
- La criptografía **no ha sido auditada**. Véase LEGAL.md.

---

## Estructura del proyecto

```
glyph/
├── Cargo.toml
├── assets/wordlists/        # listas BIP-39 vendadas (en/es/fr/it + ja/zh)
├── src/
│   ├── main.rs              # CLI (clap)
│   ├── lib.rs
│   ├── cli/                 # init_cmd, seed_cmd, code_cmd
│   ├── seed/                # prng, crypto, dialect, permute, mnemonic, shamir, wordlists
│   ├── store/               # init (almacén .glyph), layout
│   └── compiler/            # ir, op, glyph (s-expr), py (front-end)
├── tests/                   # pruebas de propiedad
└── fuzz/                    # objetivos de fuzzing
```

El almacén por proyecto vive en `.glyph/` (ya ignorado en `.gitignore`):

```
.glyph/
├── config.toml
└── dialects/
    └── <dialect_id>/
        ├── seed            # semilla en claro (modo público)
        └── seed.enc        # semilla cifrada (modo secreto)
```

> ⚠️ **Nunca subas `.glyph/` a un repositorio** salvo que la semilla esté en
> modo secreto y seas consciente de lo que compartes.

---

## Hoja de ruta

- [ ] Soporte de `ja` / `zh-hans` / `zh-hant` (normalización NFKD).
- [ ] Más lenguajes objetivo además de Python (emitters en `ir.rs`).
- [ ] `glyph build` con validación de firma del dialecto.
- [ ] Congelar el formato de cabecera/glifo y versionarlo.

---

## Licencia y avisos legales

La licencia y los descargos de responsabilidad se documentan en **LEGAL.md**.
En resumen: software sin garantía, no es asesoramiento financiero ni de
seguridad, y *Glyph Mnemonic* no es compatible con BIP-39 ni es una cartera de
criptomonedas. Las listas de palabras se vendan desde el estándar
[BIP-39](https://github.com/bitcoin/bips) (`bitcoin/bips`).

---

<p align="center">
  <sub>Glyph — ofuscación por dialecto y respaldo criptográfico para agentes y humanos.</sub>
</p>
