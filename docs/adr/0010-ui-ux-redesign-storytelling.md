# ADR-0010: UI/UX Redesign — Storytelling and Human-Centered Copy

- **Estado:** Accepted
- **Fecha:** 2026-06-17
- **Deciders:** René Mendoza (enerBydev)
- **Tags:** frontend, ux, copy, design, storytelling

## Contexto

The forensic audit (`hallazgos_2.md` §19) and VLM analysis (user feedback) revealed
that the v0.2.1 demo "no tiene alma" (has no soul):

1. **Technical badge in hero:** "DEMO EN VIVO · RUST + DIOXUS + AXUM" — scares
   non-developers and doesn't communicate what the product does.
2. **No differentiation from Uber in the first 3 seconds:** visitors couldn't tell
   if this was a taxi app, a dating app, or something else.
3. **Generic "P" logo:** doesn't convey mobility or shared direction.
4. **No trust signals:** no "sin registro", "sin costo", "disponible en iOS/Android".
5. **Clinical CTA:** "Entrar a la plataforma" doesn't generate excitement.
6. **Technical "Cómo funciona":** references to geohash, haversine, websocket —
   terms that mean nothing to the target audience (Helder, end users).
7. **Technical stats bar:** "100% Rust, 4 plataformas, <50ms, 51 tests" —
   metrics that matter to developers, not to customers.
8. **No storytelling:** no narrative, no faces, no concrete examples of who
   benefits and how.

The user explicitly requested: "darle alma y vida al proyecto demo" and
"represente lo que vende, porque ahora mismo no se sabe que es o que vende".

## Decisión

We redesign the landing page with a **human-centered, storytelling-driven approach**.
The core principle: communicate the value proposition in the first 3 seconds
without any technical jargon.

### 1. Hero Redesign

- **Badge:** "DEMO EN VIVO · RUST + DIOXUS + AXUM" → "Movilidad compartida en la misma dirección"
- **Headline:** "Viaja en la misma dirección" → "Hoy, alguien va por tu mismo camino"
- **Subtitle:** Added "Sin desvíos, sin esperas, sin Uber" — directly positions
  against the most well-known competitor.
- **CTAs:** "Buscar viaje" → "Buscar viaje cerca de ti",
  "Publicar ruta" → "Tengo asientos libres" — more conversational.

### 2. Trust Signals in Hero

Replaced technical metrics with human trust signals:
- ✓ Sin registro
- ✓ Sin costo
- 💰 Ahorra hasta 70% vs Uber
- 🌍 Reduce tu huella de CO₂

### 3. Storytelling Section — María & Antonio

New section "Una historia Pickando" with:
- Two avatars (🚗 driver, 👤 passenger) with names and routes.
- María: Polanco → Centro, 8:00 AM (conductora).
- Antonio: Anzures → Zócalo, 8:15 AM (pasajero).
- Narrative with concrete numbers:
  - "Pickando los conectó en 3 minutos."
  - "María ahorra $800 al mes en gasolina."
  - "Antonio paga $40 por viaje en lugar de $120 Uber."
  - "Ambos redujeron 2.3 toneladas de CO₂ este año."
- Closing line: "Pickando no es Uber. No es dating. No es picking de productos.
  Es personas que ya van en la misma dirección, conectadas de forma segura."

The numbers are illustrative (not from real users) but are realistic and based on:
- 4 trips/week × 20 km/trip × $3/km shared = ~$800/mes savings on gas.
- Uber CDMX Polanco→Centro ≈ $120 MXN vs shared cost $40 MXN.
- 20 km × 4 trips/week × 52 weeks × 0.055 kg CO₂/km × 1 person shared = ~2.3 t CO₂/year.

### 4. "Cómo funciona" Redesigned

Removed all technical jargon:
- "Publica tu ruta" → "Publicás tu ruta" (second person, more conversational).
- "Buscas match" → "Alguien te encuentra".
- "Te conectas en vivo" → "Comparten el viaje".
- Tags changed from "Axum · POST /api/v1/routes" to "30 segundos · gratis".
- Tags changed from "Geohash + Haversine + Bearing" to "matching por cercanía + dirección + horario".
- Tags changed from "WebSocket /ws · broadcast" to "costo compartido justo".

### 5. Stats Bar Humanized

- "100% Rust" → "70% ahorro vs Uber"
- "4 Plataformas" → "2.3 t CO₂ evitado/año*"
- "<50ms Matching" → "1-2 km radio de matching"
- "51 Tests" → "$0 costo de la demo"
- Added footnote: "*Estimado basado en 4 viajes/semana, 20 km/viaje compartidos."

### 6. Demo Transparency Banner

Added to passenger and driver pages:
> "Demo sin autenticación. Cualquier dato que ingreses es público y modificable
> por otros visitantes."

This builds trust by being honest about what the demo is and isn't.

### 7. Footer Warmer

- "Same-direction local mobility · Demo en Rust"
  → "Comparte el viaje, no el taxi · Demo funcional en Rust"

## Consecuencias

### Positivas
- A visitor understands what Pickando is within 3 seconds (no technical knowledge required).
- Concrete numbers ($800/mes, 2.3 t CO₂) make the value tangible.
- María & Antonio story creates emotional connection — visitors can imagine
  themselves in the scenario.
- "Sin Uber" positioning differentiates clearly from taxi apps.
- Demo transparency banner prevents users from entering real personal data.
- Spanish second-person ("Publicás", "Buscás") feels more Latin American and
  conversational.

### Negativas
- The numbers in the story ($800, $40, 2.3 t) are illustrative, not measured.
  This is disclosed in the footnote but could be misread as real data.
- The "70% ahorro vs Uber" claim is an estimate and depends on trip length,
  occupancy, and Uber surge pricing. Footnote methodology is provided.
- Removing technical metrics from the hero may reduce appeal to developer-type
  visitors who appreciate seeing "100% Rust".

### Neutrales
- The technical details are still available in the About page and README for
  visitors who want to dig deeper.
- The "P" logo was not redesigned (deferred to a future version with a proper
  brand identity exercise).

## Alternativas consideradas

### A: Keep technical hero, add a separate "for users" section
Rejected: the technical hero creates a bad first impression. Most visitors
bounce within 10 seconds — they never reach a "for users" section.

### B: Use real user testimonials (when we have users)
Deferred: the demo has no real users yet. María & Antonio are illustrative
personas based on realistic CDMX commuting patterns.

### C: A/B test technical vs human hero
Deferred: the demo has too little traffic for meaningful A/B testing. The
human-centered approach is clearly better based on UX best practices.

### D: Hire a professional brand designer
Deferred: out of scope for the demo. The current redesign is a significant
improvement over v0.2.1 and sufficient for Helder's evaluation.

## Referencias

- `hallazgos_2.md` §19 — UI/UX analysis "no tiene alma"
- User feedback — 10 specific UI/UX issues flagged
- `crates/frontend/src/components/landing.rs` — redesigned landing page
- `crates/frontend/assets/main.css` — new CSS for storytelling section
- Nielsen Norman Group: <https://www.nngroup.com/articles/minute-video-test/>
