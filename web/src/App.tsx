import { useState } from "react";
import { GITHUB_URL, RELEASES_URL, RUST_INSTALL_URL } from "./config";
import "./styles.css";

function GlyphMark({ size = 28 }: { size?: number }) {
  return (
    <svg
      className="mark"
      width={size}
      height={size}
      viewBox="0 0 32 32"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
    >
      <rect width="32" height="32" rx="8" fill="#17171c" />
      <path
        d="M9 22V10l14 12V10"
        stroke="#7ee0a8"
        strokeWidth="2.4"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <circle cx="23" cy="9" r="2.2" fill="#ff7759" />
    </svg>
  );
}

function GithubIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
      <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0016 8c0-4.42-3.58-8-8-8z" />
    </svg>
  );
}

function ArrowDown() {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <path d="M8 2v11M3 8l5 5 5-5" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <button
      className="copy-btn"
      onClick={() => {
        navigator.clipboard?.writeText(text);
        setCopied(true);
        setTimeout(() => setCopied(false), 1500);
      }}
    >
      {copied ? "Copiado" : "Copiar"}
    </button>
  );
}

const capabilities = [
  {
    title: "Ofuscación por dialecto",
    body: "Una semilla por proyecto deriva un dialecto único: el mismo código se ofusca distinto en cada repo. Sin la semilla, el .glf es ilegible.",
  },
  {
    title: "Cifrado real AES-256-GCM",
    body: "El cuerpo del .glf se cifra con AES-256-GCM (clave por HKDF-SHA256 desde la semilla). Confidencialidad de verdad mientras la semilla quede secreta.",
  },
  {
    title: "Respaldo multilingüe",
    body: "La semilla se codifica como frase de 24 palabras en en, es, fr e it. Cualquier frase completa, en cualquier idioma, la recupera.",
  },
  {
    title: "Shamir secret sharing",
    body: "Divide la semilla en N partes; cualquier T las reconstruye (GF(2⁸), polinomio del AES 0x1b). Comparte el control sin exponer la raíz.",
  },
  {
    title: "Transpilador round-trip",
    body: "Python → IR → glyph → Python de ida y vuelta, con round-trip idéntico en las pruebas. build y reveal invierten el proceso.",
  },
  {
    title: "100% local",
    body: "Sin dependencias de red ni telemetría. Todo ocurre en tu máquina. La semilla es la raíz de confianza y nunca se envía a ningún lado.",
  },
];

const installCmd = `cargo install --path .`;
const quickstartCmd = `# 1) Inicializa un proyecto glyph (--secret cifra la semilla)
glyph init --secret

# 2) Ofusca un .py real en un .glf cifrado
glyph obfuscate ejemplo.py        # -> ejemplo.glf

# 3) Revela el .glf de vuelta a Python idéntico
glyph reveal ejemplo.glf          # -> ejemplo.py

# 4) O compila un .glf en claro a Python
glyph build fuente.glf            # -> fuente.py`;

export default function App() {
  return (
    <>
      <div className="announcement">
        <span>Glyph está en desarrollo — binario vía cargo, sin telemetría.</span>
        <a href={GITHUB_URL}>Ver en GitHub</a>
      </div>

      <header className="nav">
        <div className="container nav-inner">
          <a className="brand" href="#top">
            <GlyphMark />
            Glyph
          </a>
          <nav className="nav-links">
            <a href="#caracteristicas">Características</a>
            <a href="#como-funciona">Cómo funciona</a>
            <a href="#instalar">Instalar</a>
            <a href={GITHUB_URL}>GitHub</a>
          </nav>
          <div className="nav-cta">
            <a className="btn btn-pill-outline" href={GITHUB_URL}>
              <GithubIcon /> Star
            </a>
            <a className="btn btn-primary" href="#instalar">
              Instalar
            </a>
          </div>
        </div>
      </header>

      <main id="top">
        <section className="hero container">
          <p className="eyebrow">CLI en Rust · agentes de IA y humanos</p>
          <h1>Ofuscación por dialecto y respaldo criptográfico.</h1>
          <p className="lead">
            Glyph compila un lenguaje intermedio compacto a código real, con
            ofuscación derivada de una semilla por proyecto y respaldo
            multilingüe (frases tipo BIP-39 + Shamir). 100% local.
          </p>
          <div className="hero-actions">
            <a className="btn btn-primary" href="#instalar">
              Instalar
            </a>
            <a className="btn btn-pill-outline" href={RELEASES_URL}>
              <ArrowDown /> Descargar
            </a>
            <a className="btn btn-ghost" href={GITHUB_URL}>
              Ir al código fuente →
            </a>
          </div>

          <div className="console" aria-label="ejemplo de uso de glyph">
            <div className="console-bar">
              <span />
              <span />
              <span />
            </div>
            <pre>
              <span className="cmd">$ glyph obfuscate ejemplo.py</span>
              {"\n"}
              <span className="tok-c">  # escribe ejemplo.glf:</span>
              {"\n"}
              <span className="tok-g">  #!glyph1 fc80080a3df7</span>
              {"\n"}
              <span className="tok-o">  ⟨blob cifrado con AES-256-GCM, ilegible sin la semilla⟩</span>
              {"\n\n"}
              <span className="cmd">$ glyph reveal ejemplo.glf</span>
              {"\n"}
              <span className="tok-c">  # reconstruye ejemplo.py idéntico al original</span>
            </pre>
          </div>
        </section>

        <section className="section container" id="caracteristicas">
          <div className="section-head">
            <p className="eyebrow">Por qué Glyph</p>
            <h2>Tres capas en un solo binario.</h2>
            <p>
              Motor de semilla y almacén, dialecto ofuscador seed-derivado y
              compilador. Todo local, sin red.
            </p>
          </div>
          <div className="grid-3">
            {capabilities.map((c) => (
              <article className="cap-card" key={c.title}>
                <div className="icon" aria-hidden="true">
                  <svg width="22" height="22" viewBox="0 0 24 24" fill="none">
                    <path
                      d="M12 2l8 4v6c0 5-3.5 8.5-8 10-4.5-1.5-8-5-8-10V6l8-4z"
                      stroke="currentColor"
                      strokeWidth="1.6"
                      strokeLinejoin="round"
                    />
                    <path d="M9 12l2 2 4-4" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" />
                  </svg>
                </div>
                <h3>{c.title}</h3>
                <p>{c.body}</p>
              </article>
            ))}
          </div>
        </section>

        <section className="container" id="como-funciona">
          <div className="band">
            <p className="eyebrow">Cómo funciona</p>
            <h2>De tu .py al .glf y de vuelta.</h2>
            <p className="band-lead">
              La semilla genera el dialecto (blake3 + permutación biyectiva de
              operadores sobre glifos Unicode). El pipeline ofusca y cifra en
              una sola tubería.
            </p>
            <div className="pipeline">
              <div className="pipe-step">
                <div className="label">entrada</div>
                <div className="name">.py</div>
              </div>
              <div className="pipe-arrow">→</div>
              <div className="pipe-step">
                <div className="label">parse</div>
                <div className="name">IR</div>
              </div>
              <div className="pipe-arrow">→</div>
              <div className="pipe-step">
                <div className="label">emit</div>
                <div className="name">glifo</div>
              </div>
              <div className="pipe-arrow">→</div>
              <div className="pipe-step">
                <div className="label">cifra</div>
                <div className="name">.glf</div>
              </div>
            </div>
          </div>
        </section>

        <section className="section container" id="instalar">
          <div className="section-head">
            <p className="eyebrow">Empezar</p>
            <h2>Instala, descarga o clona.</h2>
            <p>
              Necesitas la toolchain estable de Rust (1.74+). Sin binarios
              prefabricados: construyes desde la fuente.
            </p>
          </div>
          <div className="install-grid">
            <div className="install-card">
              <h3>Desde el código fuente</h3>
              <p>Clona el repo y construye el binario release.</p>
              <div className="code-block">
                <CopyButton text={installCmd} />
                <span className="cmt"># En el directorio del proyecto</span>
                {"\n"}
                <span className="cmd">{installCmd}</span>
                {"\n"}
                <span className="cmt"># El binario queda en target/release/glyph</span>
              </div>
            </div>

            <div className="install-card">
              <h3>Primeros pasos</h3>
              <p>Inicializa, ofusca y revela en cuatro comandos.</p>
              <div className="code-block">
                <CopyButton text={quickstartCmd} />
                {quickstartCmd.split("\n").map((line, i) => (
                  <span key={i} className={line.trim().startsWith("#") ? "cmt" : "cmd"}>
                    {line}
                    {"\n"}
                  </span>
                ))}
              </div>
            </div>
          </div>

          <div className="install-card" style={{ marginTop: 24 }}>
            <h3>¿No tienes Rust?</h3>
            <p>Instala la toolchain estable y luego construye Glyph.</p>
            <div className="dl-row">
              <a className="btn btn-primary" href={RUST_INSTALL_URL}>
                Instalar Rust
              </a>
              <a className="btn btn-pill-outline" href={RELEASES_URL}>
                <ArrowDown /> Descargar release
              </a>
              <a className="btn btn-ghost" href={GITHUB_URL}>
                Ir al repositorio de GitHub →
              </a>
            </div>
          </div>
        </section>
      </main>

      <footer className="footer">
        <div className="container footer-inner">
          <div>
            <a className="brand" href="#top">
              <GlyphMark />
              Glyph
            </a>
            <p className="blurb">
              Ofuscación por dialecto y respaldo criptográfico para agentes de
              IA y humanos.
            </p>
          </div>
          <div className="footer-col">
            <h4>Proyecto</h4>
            <a href="#caracteristicas">Características</a>
            <a href="#como-funciona">Cómo funciona</a>
            <a href="#instalar">Instalar</a>
          </div>
          <div className="footer-col">
            <h4>Recursos</h4>
            <a href={GITHUB_URL}>GitHub</a>
            <a href={RELEASES_URL}>Releases</a>
            <a href={`${GITHUB_URL}/blob/main/README.md`}>README</a>
            <a href={`${GITHUB_URL}/blob/main/LEGAL.md`}>LEGAL.md</a>
          </div>
          <div className="footer-col">
            <h4>Comunidad</h4>
            <a href={GITHUB_URL}>Issues</a>
            <a href={`${GITHUB_URL}/blob/main/LEGAL.md`}>Aviso legal</a>
          </div>
        </div>
        <div className="container footer-bottom">
          <span>© {new Date().getFullYear()} Glyph — software sin garantía.</span>
          <span>Glyph Mnemonic no es BIP-39 ni una cartera de criptomonedas.</span>
        </div>
      </footer>
    </>
  );
}
