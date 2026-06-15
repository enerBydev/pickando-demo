#!/usr/bin/env node
/**
 * Pickando — Visual Validation Pipeline
 * ======================================
 * Renders the WASM frontend in a headless Chromium browser,
 * captures screenshots of every page + mobile variants,
 * then optionally runs VLM analysis for UX/UI feedback.
 *
 * Usage:
 *   node scripts/visual-validate.js [--analyze] [--pages home,driver,passenger,about,mobile]
 *
 * Requirements:
 *   - Playwright (npm): npm install -g playwright && npx playwright install chromium
 *   - dist/ directory with pre-built WASM assets
 *   - (Optional) z-ai CLI for VLM analysis: npm install -g z-ai-web-dev-sdk
 *
 * Output:
 *   Screenshots saved to: /home/z/my-project/download/pickando-*.png
 *   VLM analyses saved to: /home/z/my-project/download/vlm-*-analysis.json
 */

const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// ── Configuration ──────────────────────────────────────────────────
const DIST_DIR = path.resolve(__dirname, '..', 'crates', 'frontend', 'dist');
const OUTPUT_DIR = '/home/z/my-project/download';
const BASE_URL = 'http://pickando.local/';
const WASM_WAIT_MS = 6000;
const PAGE_WAIT_MS = 1000;

const DESKTOP_VIEWPORT = { width: 1440, height: 900 };
const MOBILE_VIEWPORT = { width: 375, height: 812 };

const MIME_MAP = {
  html: 'text/html',
  css: 'text/css',
  js: 'application/javascript',
  wasm: 'application/wasm',
  json: 'application/json',
  png: 'image/png',
  svg: 'image/svg+xml',
};

// ── Helpers ────────────────────────────────────────────────────────

function serveLocalFile(url) {
  let localPath;
  if (url === BASE_URL || url === 'http://pickando.local') {
    localPath = path.join(DIST_DIR, 'index.html');
  } else if (url.startsWith(BASE_URL)) {
    localPath = path.join(DIST_DIR, url.replace(BASE_URL, ''));
  } else {
    return null;
  }
  localPath = localPath.split('?')[0];

  if (!fs.existsSync(localPath)) return null;

  const ext = localPath.split('.').pop();
  const contentType = MIME_MAP[ext] || 'application/octet-stream';

  if (ext === 'wasm') {
    return { contentType, body: fs.readFileSync(localPath), binary: true };
  }
  return { contentType, body: fs.readFileSync(localPath, 'utf8'), binary: false };
}

async function setupPage(browser, viewport = DESKTOP_VIEWPORT) {
  const page = await browser.newPage();
  await page.setViewportSize(viewport);

  await page.route('**/*', async (route) => {
    const url = route.request().url();
    const result = serveLocalFile(url);
    if (result) {
      route.fulfill({ contentType: result.contentType, body: result.body });
    } else {
      route.fulfill({ status: 404, contentType: 'text/plain', body: `Not found: ${url}` });
    }
  });

  return page;
}

async function capturePage(page, name, fullPage = true) {
  const outputPath = path.join(OUTPUT_DIR, `pickando-${name}.png`);
  await page.screenshot({ path: outputPath, fullPage });
  console.log(`  📸 ${name} → ${outputPath} (${(fs.statSync(outputPath).size / 1024).toFixed(1)}KB)`);
  return outputPath;
}

async function runVLMAnalysis(imagePath, prompt, outputName) {
  const outputPath = path.join(OUTPUT_DIR, `vlm-${outputName}-analysis.json`);
  try {
    execSync(
      `z-ai vision -p ${JSON.stringify(prompt)} -i ${JSON.stringify(imagePath)} -o ${JSON.stringify(outputPath)}`,
      { timeout: 120000, stdio: 'inherit' }
    );
    console.log(`  🤖 VLM → ${outputPath}`);
    return outputPath;
  } catch (e) {
    console.log(`  ⚠️  VLM failed for ${outputName}: ${e.message?.split('\n')[0]}`);
    return null;
  }
}

// ── Main Pipeline ──────────────────────────────────────────────────

async function main() {
  const args = process.argv.slice(2);
  const shouldAnalyze = args.includes('--analyze');
  const pagesArg = args.find(a => a.startsWith('--pages='));
  const requestedPages = pagesArg
    ? pagesArg.split('=')[1].split(',')
    : ['home', 'driver', 'passenger', 'about', 'mobile', 'mobile-menu'];

  console.log('🔍 Pickando Visual Validation Pipeline');
  console.log(`   Dist: ${DIST_DIR}`);
  console.log(`   Output: ${OUTPUT_DIR}`);
  console.log(`   Pages: ${requestedPages.join(', ')}`);
  console.log(`   VLM Analysis: ${shouldAnalyze ? 'ON' : 'OFF'}`);
  console.log('');

  // Verify dist exists
  if (!fs.existsSync(path.join(DIST_DIR, 'index.html'))) {
    console.error('❌ dist/index.html not found. Build the WASM frontend first.');
    process.exit(1);
  }

  const browser = await chromium.launch({ headless: true });
  const screenshots = {};

  try {
    // ── Desktop screenshots ──
    const desktopPage = await setupPage(browser, DESKTOP_VIEWPORT);
    await desktopPage.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: 15000 });
    console.log('⏳ Waiting for WASM initialization...');
    await desktopPage.waitForTimeout(WASM_WAIT_MS);

    // Verify WASM loaded
    const mainContent = await desktopPage.evaluate(() => {
      const main = document.getElementById('main');
      return main ? main.children.length : 0;
    });
    if (mainContent < 2) {
      console.log('⚠️  WASM may not have loaded correctly. Children:', mainContent);
    } else {
      console.log('✅ WASM loaded successfully. DOM children:', mainContent);
    }

    // Home
    if (requestedPages.includes('home')) {
      screenshots.home = await capturePage(desktopPage, 'landing-full');
      screenshots.homeViewport = await capturePage(desktopPage, 'landing-viewport', false);
    }

    // Driver
    if (requestedPages.includes('driver')) {
      const btn = desktopPage.locator('button:has-text("Conductor")').first();
      await btn.click();
      await desktopPage.waitForTimeout(PAGE_WAIT_MS);
      screenshots.driver = await capturePage(desktopPage, 'driver');
    }

    // Passenger
    if (requestedPages.includes('passenger')) {
      const btn = desktopPage.locator('button:has-text("Pasajero")').first();
      await btn.click();
      await desktopPage.waitForTimeout(PAGE_WAIT_MS);
      screenshots.passenger = await capturePage(desktopPage, 'passenger');
    }

    // About
    if (requestedPages.includes('about')) {
      const btn = desktopPage.locator('button:has-text("Acerca")').first();
      await btn.click();
      await desktopPage.waitForTimeout(PAGE_WAIT_MS);
      screenshots.about = await capturePage(desktopPage, 'about');
    }

    await desktopPage.close();

    // ── Mobile screenshots ──
    if (requestedPages.includes('mobile') || requestedPages.includes('mobile-menu')) {
      const mobilePage = await setupPage(browser, MOBILE_VIEWPORT);
      await mobilePage.goto(BASE_URL, { waitUntil: 'domcontentloaded', timeout: 15000 });
      await mobilePage.waitForTimeout(WASM_WAIT_MS);

      if (requestedPages.includes('mobile')) {
        screenshots.mobile = await capturePage(mobilePage, 'mobile-landing');
      }

      if (requestedPages.includes('mobile-menu')) {
        const toggle = mobilePage.locator('button.mobile-toggle').first();
        await toggle.click();
        await mobilePage.waitForTimeout(500);
        screenshots.mobileMenu = await capturePage(mobilePage, 'mobile-menu');
      }

      await mobilePage.close();
    }

    // ── VLM Analysis ──
    if (shouldAnalyze) {
      console.log('\n🤖 Running VLM analysis...');

      const prompts = {
        home: 'Evalúa esta landing page de Pickando (app de movilidad misma-dirección). Dark theme #0D0D11, acento #00FF88. Evalúa: 1) Jerarquía visual 2) Hero + QAW 3) Tipografía 4) Color 5) Cards personalidad 6) Asimetría 7) Producto real vs template. Puntuaciones 1-10.',
        driver: 'Evalúa esta página Publicar Ruta de Pickando. Flujo conversacional. Dark theme. Puntuaciones 1-10.',
        passenger: 'Evalúa esta página Buscar Viaje de Pickando. Tabs, route cards, search form. Puntuaciones 1-10.',
        mobile: 'Evalúa esta versión móvil de Pickando en iPhone 375px. Responsive, overflow, native feel. Puntuaciones 1-10.',
      };

      for (const [key, screenshotPath] of Object.entries(screenshots)) {
        const pageKey = key.replace('Viewport', '').replace('Menu', '-menu');
        const promptKey = key.replace('Viewport', '').replace('Menu', '');
        if (prompts[promptKey]) {
          await runVLMAnalysis(screenshotPath, prompts[promptKey], pageKey);
          // Rate limit: wait between requests
          await new Promise(r => setTimeout(r, 3000));
        }
      }
    }

    console.log('\n✅ Visual validation complete!');
    console.log(`   Screenshots: ${Object.keys(screenshots).length}`);
    if (shouldAnalyze) {
      const analyses = fs.readdirSync(OUTPUT_DIR).filter(f => f.startsWith('vlm-') && f.endsWith('.json'));
      console.log(`   VLM analyses: ${analyses.length}`);
    }

  } finally {
    await browser.close();
  }
}

main().catch(e => {
  console.error('❌ Pipeline failed:', e.message);
  process.exit(1);
});
