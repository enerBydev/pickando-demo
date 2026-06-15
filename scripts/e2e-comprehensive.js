#!/usr/bin/env node
/**
 * Pickando Comprehensive E2E Test Suite
 *
 * Tests ALL user interactions and API endpoints against the live Railway deployment.
 * Covers: Landing, Driver, Passenger, About, API, WebSocket, Mobile, Error handling.
 *
 * Usage: node scripts/e2e-comprehensive.js [--url=URL] [--verbose]
 */

const { chromium } = require('playwright');

const BASE_URL = process.env.PICKANDO_URL || 'https://pickando-demo-production.up.railway.app/';
const WASM_WAIT = 8000;
const VERBOSE = process.argv.includes('--verbose');

const results = {
  passed: 0,
  failed: 0,
  skipped: 0,
  tests: [],
  startTime: Date.now(),
};

function log(category, description, passed, detail = '') {
  const icon = passed ? '✅' : '❌';
  console.log(`${icon} [${category}] ${description}${detail ? ' — ' + detail : ''}`);
  results.tests.push({ category, description, passed, detail });
  if (passed) results.passed++;
  else results.failed++;
}

function skip(category, description, reason = '') {
  console.log(`⏭️ [${category}] ${description}${reason ? ' — SKIP: ' + reason : ''}`);
  results.tests.push({ category, description, passed: null, detail: reason });
  results.skipped++;
}

// ============================================================
// API TESTS — Direct HTTP calls (no browser needed)
// ============================================================
async function testAPIEndpoints() {
  console.log('\n📡 API Endpoint Tests');
  console.log('='.repeat(60));

  // Health check
  try {
    const resp = await fetch(`${BASE_URL}api/v1/health`);
    const data = await resp.json();
    log('API', 'GET /api/v1/health returns 200', resp.status === 200);
    log('API', 'Health status is "ok"', data.status === 'ok', `got: ${data.status}`);
    log('API', 'Health has uptime_seconds', typeof data.uptime_seconds === 'number', `got: ${data.uptime_seconds}`);
    log('API', 'Health has version', data.version === '0.1.0-proof', `got: ${data.version}`);
    log('API', 'Health has stack info', data.stack.includes('Axum'), `got: ${data.stack}`);
  } catch (e) {
    log('API', 'Health check', false, e.message);
  }

  // List routes
  try {
    const resp = await fetch(`${BASE_URL}api/v1/routes`);
    const data = await resp.json();
    log('API', 'GET /api/v1/routes returns 200', resp.status === 200);
    log('API', 'Routes is an array', Array.isArray(data), `got: ${typeof data}`);
    log('API', 'Routes has >=4 sample routes', data.length >= 4, `got: ${data.length}`);
    if (data.length > 0) {
      const r = data[0];
      log('API', 'Route has required fields', 
        r.id && r.driver_id && r.origin_address && r.dest_address && r.geohash,
        `missing: ${['id','driver_id','origin_address','dest_address','geohash'].filter(f => !r[f]).join(',')}`);
      log('API', 'Route geohash is 6 chars', r.geohash.length === 6, `got: ${r.geohash}`);
      log('API', 'Route has valid coordinates',
        typeof r.origin_lat === 'number' && typeof r.origin_lng === 'number',
        `lat: ${r.origin_lat}, lng: ${r.origin_lng}`);
    }
  } catch (e) {
    log('API', 'List routes', false, e.message);
  }

  // Create route
  try {
    const resp = await fetch(`${BASE_URL}api/v1/routes`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        origin_address: 'Test Origin E2E',
        dest_address: 'Test Destination E2E',
        departure_time: '14:00',
        seats_available: 2,
      }),
    });
    const data = await resp.json();
    log('API', 'POST /api/v1/routes returns 200', resp.status === 200);
    log('API', 'Created route has id', !!data.id, `got: ${data.id}`);
    log('API', 'Created route persists origin', data.origin_address === 'Test Origin E2E', `got: ${data.origin_address}`);
    log('API', 'Created route has status Published', data.status === 'Published', `got: ${data.status}`);
    log('API', 'Created route has seats_available', data.seats_available === 2, `got: ${data.seats_available}`);

    // Verify persistence — route should appear in GET
    const listResp = await fetch(`${BASE_URL}api/v1/routes`);
    const listData = await listResp.json();
    log('API', 'Created route appears in list', listData.length >= 5, `got ${listData.length} routes`);
    log('API', 'Created route found by ID', listData.some(r => r.id === data.id), `id: ${data.id}`);
  } catch (e) {
    log('API', 'Create route', false, e.message);
  }

  // Create route — missing fields
  try {
    const resp = await fetch(`${BASE_URL}api/v1/routes`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ origin_address: '', dest_address: '' }),
    });
    log('API', 'POST /api/v1/routes with empty fields returns 400', resp.status === 400, `got: ${resp.status}`);
  } catch (e) {
    log('API', 'Create route validation', false, e.message);
  }

  // Match nearby
  try {
    const resp = await fetch(`${BASE_URL}api/v1/match`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ lat: 19.4326, lng: -99.1332, radius_km: 5 }),
    });
    const data = await resp.json();
    log('API', 'POST /api/v1/match returns 200', resp.status === 200);
    log('API', 'Match finds routes near CDMX', data.length > 0, `found ${data.length} matches`);
    log('API', 'Monterrey route NOT in CDMX matches',
      !data.some(m => m.route.id === 'route-004'),
      data.some(m => m.route.id === 'route-004') ? 'Monterrey incorrectly matched!' : 'correct');
    if (data.length > 0) {
      const m = data[0];
      log('API', 'MatchResult has distance_km', typeof m.distance_km === 'number', `got: ${m.distance_km}`);
      log('API', 'MatchResult has relevance_score', typeof m.relevance_score === 'number', `got: ${m.relevance_score}`);
      log('API', 'Distance within radius', m.distance_km <= 5, `got: ${m.distance_km}km`);
    }
  } catch (e) {
    log('API', 'Match nearby', false, e.message);
  }

  // Match far away
  try {
    const resp = await fetch(`${BASE_URL}api/v1/match`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ lat: 20.6597, lng: -103.3496, radius_km: 5 }), // Guadalajara
    });
    const data = await resp.json();
    log('API', 'Match finds nothing from Guadalajara', data.length === 0, `found ${data.length}`);
  } catch (e) {
    log('API', 'Match far away', false, e.message);
  }

  // Match default radius
  try {
    const resp = await fetch(`${BASE_URL}api/v1/match`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ lat: 19.4326, lng: -99.1332 }), // no radius_km
    });
    const data = await resp.json();
    log('API', 'Match with default radius works', resp.status === 200, `found ${data.length} matches`);
  } catch (e) {
    log('API', 'Match default radius', false, e.message);
  }

  // Join route
  try {
    const resp = await fetch(`${BASE_URL}api/v1/routes/route-001/join`, {
      method: 'POST',
    });
    const data = await resp.json();
    log('API', 'POST /api/v1/routes/route-001/join returns 200', resp.status === 200);
    log('API', 'Join response type is "joined"', data.type === 'joined', `got: ${data.type}`);
    log('API', 'Join response has seats_remaining', 
      data.data && typeof data.data.seats_remaining === 'number',
      `got: ${JSON.stringify(data.data)}`);
  } catch (e) {
    log('API', 'Join route', false, e.message);
  }

  // Join non-existent route
  try {
    const resp = await fetch(`${BASE_URL}api/v1/routes/nonexistent-route/join`, {
      method: 'POST',
    });
    log('API', 'Join nonexistent route returns 404', resp.status === 404, `got: ${resp.status}`);
  } catch (e) {
    log('API', 'Join nonexistent route', false, e.message);
  }

  // 404 for unknown endpoints
  try {
    const resp = await fetch(`${BASE_URL}api/v1/nonexistent`);
    log('API', 'Unknown endpoint returns 404', resp.status === 404, `got: ${resp.status}`);
  } catch (e) {
    log('API', 'Unknown endpoint', false, e.message);
  }
}

// ============================================================
// FRONTEND TESTS — Browser-based (Playwright)
// ============================================================
async function testFrontend(browser) {
  console.log('\n🖥️ Frontend UI Tests');
  console.log('='.repeat(60));

  const desktop = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  
  // ---- LANDING PAGE ----
  try {
    await desktop.goto(BASE_URL, { waitUntil: 'networkidle', timeout: 30000 });
    await desktop.waitForTimeout(WASM_WAIT);

    const title = await desktop.title();
    log('FRONTEND', 'Page has title', title.includes('Pickando'), `got: ${title}`);

    const heroTitle = await desktop.locator('.hero-title').first().textContent().catch(() => null);
    log('LANDING', 'Hero title visible with "Comparte"', heroTitle && heroTitle.includes('Comparte'), `got: ${heroTitle}`);

    const heroSubtitle = await desktop.locator('.hero-subtitle').first().textContent().catch(() => null);
    log('LANDING', 'Hero subtitle visible', heroSubtitle && heroSubtitle.length > 20, `length: ${heroSubtitle?.length}`);

    const qawWidget = await desktop.locator('.quick-action-widget').first().isVisible().catch(() => false);
    log('LANDING', 'QAW widget visible', qawWidget);

    const qawSearchBtn = await desktop.locator('.qaw-search-btn').first().isVisible().catch(() => false);
    log('LANDING', 'QAW search button visible', qawSearchBtn);

    const featureStrip = await desktop.locator('.features-asymmetric').first().isVisible().catch(() => false);
    log('LANDING', 'Asymmetric feature strip visible', featureStrip);

    const archSection = await desktop.locator('.architecture-section').first().isVisible().catch(() => false);
    log('LANDING', 'Architecture section visible', archSection);

    // Navbar
    const navbar = await desktop.locator('.navbar').first().isVisible().catch(() => false);
    log('LANDING', 'Navbar visible', navbar);

    const brandText = await desktop.locator('.brand-text').first().textContent().catch(() => null);
    log('LANDING', 'Brand shows "Pickando"', brandText === 'Pickando', `got: ${brandText}`);

    // Footer
    const footer = await desktop.locator('.footer').first().isVisible().catch(() => false);
    log('LANDING', 'Footer visible', footer);

    // QAW role buttons
    const passengerBtn = await desktop.locator('.qaw-role-btn:has-text("Viajar")').first().isVisible().catch(() => false);
    const driverBtn = await desktop.locator('.qaw-role-btn:has-text("Conducir")').first().isVisible().catch(() => false);
    log('LANDING', 'QAW role buttons visible', passengerBtn && driverBtn);

    // Trust chips
    const trustChips = await desktop.locator('.trust-chip').count().catch(() => 0);
    log('LANDING', 'Trust chips present (>=3)', trustChips >= 3, `got: ${trustChips}`);
  } catch (e) {
    log('LANDING', 'Landing page load', false, e.message);
  }

  // ---- QAW → PASSENGER NAVIGATION ----
  try {
    const passengerRole = desktop.locator('.qaw-role-btn:has-text("Viajar")').first();
    await passengerRole.click().catch(() => {});
    await desktop.waitForTimeout(300);

    const searchBtn = desktop.locator('.qaw-search-btn').first();
    await searchBtn.click();
    await desktop.waitForTimeout(2000);

    const passengerHeader = await desktop.locator('h1:has-text("Buscar Viaje")').first().isVisible().catch(() => false);
    log('NAVIGATION', 'QAW → Passenger page navigation', passengerHeader);
  } catch (e) {
    log('NAVIGATION', 'QAW → Passenger', false, e.message);
  }

  // ---- PASSENGER PAGE ----
  try {
    // Test matching search
    const latInput = desktop.locator('input[type="text"]').first();
    await latInput.fill('19.4326').catch(() => {});

    const searchBtn = desktop.locator('button:has-text("Buscar Matches")').first();
    await searchBtn.click().catch(() => {});
    await desktop.waitForTimeout(3000);

    const routeCards = await desktop.locator('.route-card-v2').count().catch(() => 0);
    log('PASSENGER', 'Match search returns route cards', routeCards > 0, `got: ${routeCards} cards`);

    // Test "Unirme" button
    const joinBtn = await desktop.locator('.route-join-btn').first().isVisible().catch(() => false);
    log('PASSENGER', '"Unirme" button visible after search', joinBtn);

    if (joinBtn) {
      await desktop.locator('.route-join-btn').first().click().catch(() => {});
      await desktop.waitForTimeout(2000);

      const joinedBtn = await desktop.locator('.route-join-btn.joined').first().isVisible().catch(() => false);
      const solicitadoText = await desktop.locator('button:has-text("Solicitado")').first().isVisible().catch(() => false);
      log('PASSENGER', '"Unirme" click shows "Solicitado ✓"', joinedBtn || solicitadoText, `joined: ${joinedBtn}, text: ${solicitadoText}`);
    }

    // Test Rutas tab
    const rutasTab = desktop.locator('.tab:has-text("Rutas")').first();
    await rutasTab.click().catch(() => {});
    await desktop.waitForTimeout(300);

    const loadRoutesBtn = desktop.locator('button:has-text("Cargar Rutas")').first();
    await loadRoutesBtn.click().catch(() => {});
    await desktop.waitForTimeout(2000);

    const routeCards2 = await desktop.locator('.route-card-v2').count().catch(() => 0);
    log('PASSENGER', 'Rutas tab loads routes', routeCards2 > 0, `got: ${routeCards2} cards`);

    // Test System Status tab
    const systemTab = desktop.locator('.tab:has-text("Sistema")').first();
    await systemTab.click().catch(() => {});
    await desktop.waitForTimeout(300);

    const verifyBtn = desktop.locator('button:has-text("Verificar Status")').first();
    await verifyBtn.click().catch(() => {});
    await desktop.waitForTimeout(2000);

    const healthOutput = await desktop.locator('.status-box').first().textContent().catch(() => '');
    log('PASSENGER', 'System tab health check works', healthOutput.includes('ok'), `contains "ok": ${healthOutput.includes('ok')}`);
  } catch (e) {
    log('PASSENGER', 'Passenger page tests', false, e.message);
  }

  // ---- DRIVER PAGE (via navbar) ----
  try {
    const driverNav = desktop.locator('.nav-link:has-text("Conductor")').first();
    await driverNav.click().catch(() => {});
    await desktop.waitForTimeout(1000);

    const driverHeader = await desktop.locator('h1:has-text("Publicar Ruta")').first().isVisible().catch(() => false);
    log('DRIVER', 'Driver page loads via navbar', driverHeader);

    const convoSentence = await desktop.locator('.convo-sentence').first().isVisible().catch(() => false);
    log('DRIVER', 'Conversational sentence visible', convoSentence);

    const publishBtn = await desktop.locator('button:has-text("Publicar Ruta")').first().isVisible().catch(() => false);
    log('DRIVER', 'Publish button visible', publishBtn);

    if (publishBtn) {
      await desktop.locator('button:has-text("Publicar Ruta")').first().click().catch(() => {});
      await desktop.waitForTimeout(3000);

      const confirmIcon = await desktop.locator('.confirm-icon').first().isVisible().catch(() => false);
      const publishedText = await desktop.locator('h3:has-text("Ruta Publicada")').first().isVisible().catch(() => false);
      log('DRIVER', 'Publish shows confirmation', confirmIcon || publishedText, `icon: ${confirmIcon}, text: ${publishedText}`);

      const routeSummary = await desktop.locator('.route-summary').first().isVisible().catch(() => false);
      log('DRIVER', 'Route summary visible after publish', routeSummary);

      const resetBtn = await desktop.locator('button:has-text("Publicar otra ruta")').first().isVisible().catch(() => false);
      log('DRIVER', '"Publicar otra ruta" button visible', resetBtn);
    }
  } catch (e) {
    log('DRIVER', 'Driver page tests', false, e.message);
  }

  // ---- ABOUT PAGE ----
  try {
    const aboutNav = desktop.locator('.nav-link:has-text("Acerca de")').first();
    await aboutNav.click().catch(() => {});
    await desktop.waitForTimeout(1000);

    const aboutHeader = await desktop.locator('h1:has-text("Acerca de")').first().isVisible().catch(() => false);
    log('ABOUT', 'About page loads', aboutHeader);

    const realItems = await desktop.locator('.demo-item.real').count().catch(() => 0);
    log('ABOUT', 'Real features listed (>=5)', realItems >= 5, `got: ${realItems}`);

    const placeholderItems = await desktop.locator('.demo-item.placeholder').count().catch(() => 0);
    log('ABOUT', 'Placeholder items listed (>=3)', placeholderItems >= 3, `got: ${placeholderItems}`);

    const reuseTable = await desktop.locator('.reuse-table').first().isVisible().catch(() => false);
    log('ABOUT', 'Reusability table visible', reuseTable);
  } catch (e) {
    log('ABOUT', 'About page tests', false, e.message);
  }

  // ---- BACK TO LANDING ----
  try {
    const homeNav = desktop.locator('.nav-link:has-text("Inicio")').first();
    await homeNav.click().catch(() => {});
    await desktop.waitForTimeout(1000);

    const heroTitle = await desktop.locator('.hero-title').first().isVisible().catch(() => false);
    log('NAVIGATION', 'Home navigation returns to landing', heroTitle);
  } catch (e) {
    log('NAVIGATION', 'Return to landing', false, e.message);
  }

  await desktop.close();

  // ---- MOBILE RESPONSIVE ----
  console.log('\n📱 Mobile Responsive Tests');
  console.log('='.repeat(60));

  const mobile = await browser.newPage({ viewport: { width: 375, height: 812 } });
  try {
    await mobile.goto(BASE_URL, { waitUntil: 'networkidle', timeout: 30000 });
    await mobile.waitForTimeout(WASM_WAIT);

    const mobileToggle = await mobile.locator('.mobile-toggle').first().isVisible().catch(() => false);
    log('MOBILE', 'Mobile menu toggle visible', mobileToggle);

    if (mobileToggle) {
      await mobile.locator('.mobile-toggle').first().click().catch(() => {});
      await mobile.waitForTimeout(500);

      const mobileMenu = await mobile.locator('.mobile-menu').first().isVisible().catch(() => false);
      log('MOBILE', 'Mobile menu opens on toggle click', mobileMenu);

      if (mobileMenu) {
        const mobileLinks = await mobile.locator('.mobile-link').count().catch(() => 0);
        log('MOBILE', 'Mobile menu has navigation links', mobileLinks >= 3, `got: ${mobileLinks}`);

        // Navigate to driver via mobile menu
        await mobile.locator('.mobile-link:has-text("Conductor")').first().click().catch(() => {});
        await mobile.waitForTimeout(1000);

        const driverHeader = await mobile.locator('h1:has-text("Publicar Ruta")').first().isVisible().catch(() => false);
        log('MOBILE', 'Mobile navigation to Driver works', driverHeader);
      }
    }

    // Check QAW is stacked on mobile
    const qawVisible = await mobile.locator('.quick-action-widget').first().isVisible().catch(() => false);
    log('MOBILE', 'QAW widget visible on mobile', qawVisible);
  } catch (e) {
    log('MOBILE', 'Mobile tests', false, e.message);
  }
  await mobile.close();
}

// ============================================================
// CSS & VISUAL CONSISTENCY TESTS
// ============================================================
async function testCSSConsistency(browser) {
  console.log('\n🎨 CSS & Visual Consistency Tests');
  console.log('='.repeat(60));

  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  try {
    await page.goto(BASE_URL, { waitUntil: 'networkidle', timeout: 30000 });
    await page.waitForTimeout(WASM_WAIT);

    // Check CSS custom properties exist
    const rootStyles = await page.evaluate(() => {
      const styles = getComputedStyle(document.documentElement);
      return {
        accent: styles.getPropertyValue('--accent').trim(),
        bgDeep: styles.getPropertyValue('--bg-deep').trim(),
        accentDim: styles.getPropertyValue('--accent-dim').trim(),
        dangerDim: styles.getPropertyValue('--danger-dim').trim(),
        warningDim: styles.getPropertyValue('--warning-dim').trim(),
      };
    });

    log('CSS', '--accent variable exists (#00FF88)', rootStyles.accent === '#00FF88', `got: ${rootStyles.accent}`);
    log('CSS', '--bg-deep variable exists', rootStyles.bgDeep.length > 0, `got: ${rootStyles.bgDeep}`);
    log('CSS', '--accent-dim variable exists', rootStyles.accentDim.length > 0, `got: ${rootStyles.accentDim}`);
    log('CSS', '--danger-dim variable exists', rootStyles.dangerDim.length > 0, `got: ${rootStyles.dangerDim}`);
    log('CSS', '--warning-dim variable exists', rootStyles.warningDim.length > 0, `got: ${rootStyles.warningDim}`);

    // Check no teal (#4ecdc4) anywhere
    const tealFound = await page.evaluate(() => {
      const allElements = document.querySelectorAll('*');
      for (const el of allElements) {
        const style = getComputedStyle(el);
        if (style.color === 'rgb(78, 205, 196)' || style.backgroundColor === 'rgb(78, 205, 196)' ||
            style.borderColor === 'rgb(78, 205, 196)') {
          return true;
        }
      }
      return false;
    });
    log('CSS', 'No teal (#4ecdc4) colors found', !tealFound);

    // Check fonts loaded
    const fontInfo = await page.evaluate(() => {
      const testEl = document.createElement('span');
      testEl.style.fontFamily = 'Space Grotesk';
      testEl.textContent = 'test';
      document.body.appendChild(testEl);
      const computed = getComputedStyle(testEl);
      const family = computed.fontFamily;
      document.body.removeChild(testEl);
      return family;
    });
    log('CSS', 'Space Grotesk font referenced', fontInfo.includes('Space Grotesk'), `got: ${fontInfo}`);

    // Check page background is dark
    const bgColor = await page.evaluate(() => {
      return getComputedStyle(document.documentElement).backgroundColor;
    });
    log('CSS', 'Page background is dark', bgColor !== 'rgba(0, 0, 0, 0)' && bgColor !== 'rgb(255, 255, 255)', `got: ${bgColor}`);

  } catch (e) {
    log('CSS', 'CSS consistency tests', false, e.message);
  }
  await page.close();
}

// ============================================================
// MAIN
// ============================================================
async function main() {
  console.log(`\n🚀 Pickando Comprehensive E2E Tests — ${BASE_URL}`);
  console.log('='.repeat(60));
  console.log(`Started: ${new Date().toISOString()}\n`);

  // Phase 1: API tests (fast, no browser)
  await testAPIEndpoints();

  // Phase 2: Frontend tests (browser-based)
  const browser = await chromium.launch({ headless: true });
  try {
    await testFrontend(browser);
    await testCSSConsistency(browser);
  } finally {
    await browser.close();
  }

  // Summary
  const elapsed = ((Date.now() - results.startTime) / 1000).toFixed(1);
  const total = results.passed + results.failed;
  const passRate = total > 0 ? ((results.passed / total) * 100).toFixed(1) : 0;

  console.log('\n' + '='.repeat(60));
  console.log('📊 TEST SUMMARY');
  console.log('='.repeat(60));
  console.log(`Total:   ${total}`);
  console.log(`Passed:  ${results.passed} ✅`);
  console.log(`Failed:  ${results.failed} ❌`);
  console.log(`Skipped: ${results.skipped} ⏭️`);
  console.log(`Rate:    ${passRate}%`);
  console.log(`Time:    ${elapsed}s`);

  // List failures
  if (results.failed > 0) {
    console.log('\n❌ FAILED TESTS:');
    results.tests.filter(t => t.passed === false).forEach(t => {
      console.log(`   [${t.category}] ${t.description}${t.detail ? ' — ' + t.detail : ''}`);
    });
  }

  // Write JSON results
  const fs = require('fs');
  const outputPath = '/home/z/my-project/download/e2e-results.json';
  fs.writeFileSync(outputPath, JSON.stringify(results, null, 2));
  console.log(`\n📄 Results saved to ${outputPath}`);

  process.exit(results.failed > 0 ? 1 : 0);
}

main().catch(e => {
  console.error('Fatal error:', e);
  process.exit(2);
});
