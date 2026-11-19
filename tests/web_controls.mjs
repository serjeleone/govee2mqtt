import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const html = (strings, ...values) => ({ strings, values });
const textOf = value => {
  if (Array.isArray(value)) return value.map(textOf).join('');
  if (value?.strings) return value.strings.reduce((result, text, i) => result + text + textOf(value.values[i]), '');
  return value == null || typeof value === 'function' ? '' : String(value);
};
globalThis.__goveeTest = { LitElement: class {}, html, Task: class {} };
globalThis.customElements = { define() {} };
globalThis.window = { gvToast() {} };
let source = await readFile(new URL('../assets/components/devices.js', import.meta.url), 'utf8');
source = source.replace(/^import .*;\n/gm, '');
source = `const {LitElement, html, Task} = globalThis.__goveeTest; const timeAgo = () => '';\n${source}`;
const { DeviceList } = await import(`data:text/javascript;base64,${Buffer.from(source).toString('base64')}`);

test('ScenicView buttons target the saved Platform object, with no active-state switch', () => {
  const list = new DeviceList();
  const item = { id: '12131030', safe_id: '12131030', name: 'ScenicView', sku: 'DreamViewScenic',
    is_virtual: true, virtual_kind: 'dream_view_scene', available: true, state: null };
  const view = textOf(list._render_item(item));
  assert.match(view, /data-path=api\/device/);
  assert.match(view, /data-action=power\/on/);
  assert.match(view, /data-action=power\/off/);
  assert.doesNotMatch(view, /role="switch"/);
});

test('physical video center keeps power separate from DreamView command buttons', () => {
  const list = new DeviceList();
  list.scenesCache.AABB = [];
  const item = { safe_id: 'AABB', name: 'TV Backlight', sku: 'R2A80', is_virtual: false,
    supports_dreamview: true, state: null };
  const view = textOf(list._render_item(item));
  assert.match(view, /role="switch"/);
  assert.match(view, /data-action=dreamview\/on/);
  assert.match(view, /data-action=dreamview\/off/);
});

test('click sends an explicit command even when the clicked child has no data attributes', async () => {
  const list = new DeviceList();
  const requests = [];
  globalThis.fetch = async (url, options) => { requests.push({ url, options }); return { ok: true }; };
  const button = { dataset: { id: 'AA:BB', name: 'TV', path: 'api/device', action: 'dreamview/off' }, disabled: false };
  await list._sendCommand({ target: {}, currentTarget: button, stopPropagation() {} });
  assert.deepEqual(requests, [{ url: 'api/device/AA%3ABB/dreamview/off', options: { method: 'POST' } }]);
  assert.equal(button.disabled, false);
});

test('a rejected cloud command shows the API error without a success toast', async () => {
  const messages = [];
  globalThis.window.gvToast = (...args) => messages.push(args);
  globalThis.fetch = async () => ({ ok: false, status: 500, json: async () => ({ msg: 'devices not exist' }) });
  await new DeviceList()._apiAction('api/device/12131030/power/on', 'ScenicView On command accepted');
  assert.deepEqual(messages, [['devices not exist', 'error']]);
});
