# LEGAL.md — Avisos y descargos de responsabilidad (Glyph)

> **Resumen:** Glyph es software libre sin garantía. No es asesoramiento
> financiero ni de seguridad. La criptografía no ha sido auditada. *Glyph
> Mnemonic* **no es BIP-39** y **no es una cartera de criptomonedas**.

## 1. Sin garantía

El software se proporciona **"TAL CUAL"** (AS IS), sin garantías de ningún
tipo, expresas o implícitas, incluyendo —sin limitación— las garantías de
comercialización, idoneidad para un propósito particular o no infracción. El
autor y los colaboradores no responden de daños directos, indirectos,
incidentales, especiales, consecuentes o punitivos que surjan del uso o la
imposibilidad de uso del software.

## 2. No es asesoramiento de seguridad ni financiero

Glyph es una herramienta de **ofuscación y respaldo** pensada para agentes de
IA y desarrolladores. No constituye asesoramiento de seguridad, criptográfico
ni financiero. No lo uses como único mecanismo de protección de secretos
críticos, ni como sustituto de un gestor de contraseñas auditado o de un
módulo de hardware seguro (HSM / cartera de hardware).

## 3. Aviso de criptografía

- Glyph usa **AES-256-GCM** (via `aes-gcm`) y derivación de clave **HKDF-SHA256**
  (via `hkdf`) a partir de la semilla del proyecto.
- El código **no ha sido sometido a una auditoría de seguridad independiente**.
  Aunque las pruebas de round-trip y de propiedad pasan, la correctitud
  criptográfica en escenarios adversos no está garantizada. Úsalo bajo tu
  propio riesgo y, si vas a proteger algo importante, encarga una revisión
  externa.
- La ofuscación por dialecto (glifos) es una medida de **opacidad**, no de
  seguridad fuerte. No presupongas que un `.glf` ofuscado resiste a un atacante
  que ya posee la semilla o que puede deducir el dialecto.

## 4. Glyph Mnemonic no es BIP-39

El formato de frase de recuperación de Glyph (*Glyph Mnemonic v1*) es
**independiente** del estándar BIP-39:

- Codifica 256 bits de entropía + 8 bits de checksum SHA-256 = 24 palabras
  (estilo BIP-39), pero la semilla resultante **no es compatible** con carteras
  BIP-39/BIP-32.
- **No introduzcas una frase de Glyph en una cartera de criptomonedas**, ni
  esperes recuperar fondos con ella. No está asociada a ningún activo
  financiero.
- Las mismas 256 bits se expresan como frases equivalentes en varios idiomas
  (`en`, `es`, `fr`, `it`); cualquier frase completa recupera la semilla.

## 5. Atribución de las listas de palabras

Las listas de palabras se **vendan** (se incluyen en `assets/wordlists/`) desde
el estándar [**BIP-39**](https://github.com/bitcoin/bips) del repositorio
`bitcoin/bips`. Dichas listas se distribuyen bajo los términos del propio
repositorio (MIT). El archivo `assets/wordlists/manifest.toml` registra el
SHA-256 de cada lista vendada.

Los idiomas `ja` (japonés), `zh-hans` y `zh-hant` se incluyen en los archivos
pero **están deshabilitados** en esta versión (requieren normalización NFKD y
manejo de espacios ideográficos); no se usarán para codificar semillas hasta
que se habilite explícitamente.

## 6. Glifos Unicode

El alfabeto de glifos (`GLYPH_SOURCE`) usa caracteres Unicode (símbolos
matemáticos/tecnológicos). No se distribuye ni requiere ninguna fuente
tipográfica de terceros: se renderiza con las fuentes del sistema. No hay
marca registrada ni recurso con licencia asociado a dichos glifos más allá de
lo cubierto por el estándar Unicode.

## 7. Limitación de responsabilidad

En la medida máxima permitida por la ley, el autor no será responsable ante ti
ni ante nadie más por cualquier reclamación, daño o pérdida (incluyendo pérdida
de datos o de acceso a tus propios secretos por olvido de passphrase o frase de
recuperación) derivados del uso de Glyph. **Eres responsable de guardar tu
semilla, tu passphrase y tus frases de recuperación en un lugar seguro.** Si
pierdes la semilla y todas las frases/partes de Shamir, los datos ofuscados
serán irrecuperables.

## 8. Licencia del código

El código fuente de Glyph se distribuye bajo los términos indicados en el
repositorio (véase `README.md` y el campo de licencia del proyecto). Si no se
especifica otra licencia en `Cargo.toml` o en un archivo `LICENSE`, el código
se entrega bajo la licencia MIT salvo que se indique lo contrario.

---

<p align="center"><sub>Última actualización: véase el historial del repositorio.</sub></p>
