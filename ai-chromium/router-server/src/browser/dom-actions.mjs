export function buildWaitForEditorExpression(isGemini) {
  const findExpr = isGemini
    ? `document.querySelector('div[contenteditable="true"]') || document.querySelector('rich-textarea div[contenteditable="true"]') || document.querySelector('textarea')`
    : `document.querySelector('div[contenteditable="true"]') || document.querySelector('textarea')`;
  return `
(async () => {
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const deadline = Date.now() + 45000;
  const find = () => ${findExpr};
  while (!find() && Date.now() < deadline) await sleep(250);
  return Boolean(find());
})()`;
}

export function buildSubmitExpression(prompt, { isGemini, isGroupChat }) {
  const promptJson = JSON.stringify(prompt);
  const findEditorExpr = isGemini
    ? `document.querySelector('div[contenteditable="true"]') || document.querySelector('rich-textarea div[contenteditable="true"]') || document.querySelector('textarea')`
    : `document.querySelector('div[contenteditable="true"]') || document.querySelector('textarea')`;
  const submitExpr = isGroupChat
    ? `const enterEvent = { key: 'Enter', code: 'Enter', which: 13, keyCode: 13, bubbles: true, cancelable: true };
  editor.dispatchEvent(new KeyboardEvent('keydown', enterEvent));
  editor.dispatchEvent(new KeyboardEvent('keypress', enterEvent));
  editor.dispatchEvent(new KeyboardEvent('keyup', enterEvent));
  return { ok: true, method: "enter_group_chat" };`
    : `const send = findSend();
  if (send && !send.disabled) { send.click(); return { ok: true, method: "button" }; }
  editor.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', code: 'Enter', which: 13, keyCode: 13, bubbles: true, cancelable: true }));
  return { ok: true, method: "enter_fallback" };`;

  return `
(async () => {
  const prompt = ${promptJson};
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const deadline = Date.now() + 30000;
  const findEditor = () => ${findEditorExpr};
  while (!findEditor() && Date.now() < deadline) await sleep(250);
  const editor = findEditor();
  if (!editor) return { ok: false, error: "editor_not_found" };
  editor.focus();
  try {
    document.execCommand('selectAll', false, null);
    document.execCommand('insertText', false, prompt);
  } catch {}
  if ((editor.textContent || editor.value || '').trim().length === 0) {
    if ('value' in editor) editor.value = prompt;
    else editor.textContent = prompt;
  }
  editor.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertText', data: prompt }));
  editor.dispatchEvent(new Event('change', { bubbles: true }));
  await sleep(300);
  const findSend = () => {
    const direct = document.querySelector('button[data-testid="send-button"]') ||
      document.querySelector('button[aria-label="Send prompt"]') ||
      document.querySelector('button[aria-label="Send message"]') ||
      document.querySelector('button[aria-label="Submit"]');
    if (direct && !direct.disabled) return direct;
    const buttons = Array.from(document.querySelectorAll('button'));
    return buttons.find((b) => !b.disabled && /send|submit/i.test(b.getAttribute('aria-label') || b.textContent || '')) || null;
  };
  while (!findSend() && Date.now() < deadline) await sleep(100);
  ${submitExpr}
})()`;
}

export function buildPrepareGroupChatExpression() {
  return `
(() => {
  const norm = (s) => (s || '').replace(/\\s+/g, ' ').trim().toLowerCase();
  const visible = (el) => {
    const st = getComputedStyle(el);
    const r = el.getBoundingClientRect();
    return st.display !== 'none' && st.visibility !== 'hidden' && r.width > 0 && r.height > 0;
  };
  const clickish = (el) => {
    try { el.scrollIntoView({ block: 'center', inline: 'center' }); } catch {}
    const r = el.getBoundingClientRect();
    const cx = r.left + r.width / 2;
    const cy = r.top + r.height / 2;
    const t = document.elementFromPoint(cx, cy) || el;
    const fire = (node, type) => node.dispatchEvent(new MouseEvent(type, { bubbles: true, cancelable: true, clientX: cx, clientY: cy }));
    fire(t, 'mousedown'); fire(t, 'mouseup'); fire(t, 'click');
    try { el.click(); } catch {}
  };

  const href = String(location.href || '');
  if (!href.includes('/gg/')) return { ok: true, skipped: true };

  const editor = document.querySelector('div[contenteditable="true"]') || document.querySelector('textarea');
  const sendBtn = document.querySelector('button[data-testid="send-button"]') ||
    document.querySelector('button[aria-label="Send prompt"]') ||
    document.querySelector('button[aria-label="Send message"]');
  if (editor && sendBtn && !sendBtn.disabled) return { ok: true, ready: true };

  const controls = Array.from(document.querySelectorAll('button,[role="button"],[role="menuitem"],a')).filter(visible);
  const createBtn = controls.find((el) => {
    const text = norm(el.innerText || el.textContent);
    const aria = norm(el.getAttribute('aria-label') || '');
    const all = (text + ' ' + aria).trim();
    return all.includes('create group') || all.includes('start group') || all === 'create' || all === 'start';
  });
  if (createBtn && !createBtn.disabled) {
    clickish(createBtn);
    return { ok: true, clicked_create: true };
  }

  const inviteField = Array.from(document.querySelectorAll('input,textarea,[contenteditable="true"]')).find((el) => {
    if (!visible(el)) return false;
    const ph = norm(el.getAttribute?.('placeholder') || '');
    const aria = norm(el.getAttribute?.('aria-label') || '');
    const all = (ph + ' ' + aria).trim();
    return all.includes('invite') || all.includes('add people') || all.includes('add members');
  });
  if (inviteField) return { ok: false, needs_manual_setup: true, reason: 'group_chat_requires_participants' };

  return { ok: true, uncertain: true };
})()`;
}
