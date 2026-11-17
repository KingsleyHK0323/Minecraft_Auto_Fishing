const { test } = require('node:test');
const assert = require('node:assert/strict');
const { spawnSync } = require('node:child_process');
const path = require('node:path');
const script = path.join(__dirname, '../scripts/smoke-test.js');
function run(site) {
  return spawnSync(process.execPath, [script, 'deploy'], {
    env: { ...process.env, SITE_URL: site, EXPECTED_VERSION: '' }, encoding: 'utf8'
  });
}
test('deploy accepts the current Pages URL and continues to version validation', () => {
  const result = run('https://kingsleyhk0323.github.io/Minecraft_Auto_Fishing/');
  assert.equal(result.status, 1);
  assert.match(result.stderr, /EXPECTED_VERSION must be a git SHA/);
});
test('deploy rejects an unrelated Pages URL before making requests', () => {
  const result = run('https://example.github.io/Minecraft_Auto_Fishing/');
  assert.equal(result.status, 1);
  assert.match(result.stderr, /SITE_URL must be exactly/);
});
// Updated by: OpenAI-Codex_Website
