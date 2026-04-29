import { CdpSocket } from "../browser/cdp-socket.mjs";

function sleep(ms) { return new Promise((r) => setTimeout(r, ms)); }

export function createGroupChatActionHandler({ parseJsonObject, readBody, errorResponse, jsonResponse, targetManager }) {
  return async function handleCreateGroupChat(req, res) {
    let body;
    try { body = parseJsonObject(await readBody(req)); }
    catch (err) { errorResponse(res, 400, err.message, { code: "invalid_request" }); return; }

    const targetUrl = String(body.target_url ?? "https://chatgpt.com/");
    const target = await targetManager.newTarget(targetUrl);
    if (!target?.webSocketDebuggerUrl) {
      errorResponse(res, 502, "CDP target has no webSocketDebuggerUrl", { code: "cdp_target_invalid" });
      return;
    }
    if (target.id) await targetManager.activateTarget(target.id);

    const cdp = new CdpSocket(target.webSocketDebuggerUrl);
    try {
      await cdp.connect();
      await cdp.send("Page.enable");
      await cdp.send("Runtime.enable");
      await cdp.send("Page.navigate", { url: "https://chatgpt.com/" });
      await sleep(1800);

      const step = await cdp.send("Runtime.evaluate", {
        expression: `
(async () => {
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const norm = (s) => (s || '').replace(/\\s+/g, ' ').trim().toLowerCase();
  const visible = (el) => {
    const st = getComputedStyle(el);
    const r = el.getBoundingClientRect();
    return st.display !== 'none' && st.visibility !== 'hidden' && r.width > 0 && r.height > 0;
  };
  const clickish = (el) => {
    try { el.scrollIntoView({ block: 'center', inline: 'center' }); } catch {}
    const r = el.getBoundingClientRect();
    const cx = r.left + r.width / 2; const cy = r.top + r.height / 2;
    const t = document.elementFromPoint(cx, cy) || el;
    const fire = (node, type) => node.dispatchEvent(new MouseEvent(type, { bubbles: true, cancelable: true, clientX: cx, clientY: cy }));
    fire(t, 'mousedown'); fire(t, 'mouseup'); fire(t, 'click');
    try { el.click(); } catch {}
  };
  const startHref = String(location.href || '');

  const controls0 = Array.from(document.querySelectorAll('button,[role="button"],a')).filter(visible);
  const start = document.querySelector('button[aria-label="Start a group chat"]') ||
    controls0.find((el) => {
      const text = norm(el.innerText || el.textContent);
      const aria = norm(el.getAttribute('aria-label') || '');
      const all = (text + ' ' + aria).trim();
      return all.includes('start a group chat') || all.includes('start group chat') || all.includes('group chat');
    });
  if (!start) return { ok: false, reason: "start_group_chat_button_missing", href: location.href };
  clickish(start);
  const deadline = Date.now() + 8000;
  let clickedStartGroup = false;
  let finalHref = startHref;
  while (Date.now() < deadline) {
    await sleep(150);
    const controls = Array.from(document.querySelectorAll('button,[role="button"],a')).filter(visible);
    const startGroup = controls.find((el) => norm(el.innerText || el.textContent).includes('start group chat'));
    if (startGroup && !clickedStartGroup) { clickish(startGroup); clickedStartGroup = true; await sleep(250); }
    const hrefNow = String(location.href || '');
    if (hrefNow && hrefNow !== startHref && hrefNow.startsWith('https://chatgpt.com/')) {
      // Close post-create modal (e.g. "Copy link") so composer is usable.
      const closeCandidates = Array.from(document.querySelectorAll('button,[role="button"],a')).filter(visible);
      const closeBtn = closeCandidates.find((el) => {
        const text = norm(el.innerText || el.textContent);
        const aria = norm(el.getAttribute('aria-label') || '');
        const all = (text + ' ' + aria).trim();
        return all === 'close' || all.includes('close') || all.includes('dismiss') || all === 'x';
      });
      if (closeBtn) {
        clickish(closeBtn);
      } else {
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape', bubbles: true }));
        document.dispatchEvent(new KeyboardEvent('keyup', { key: 'Escape', code: 'Escape', bubbles: true }));
      }
      return { ok: true, href: hrefNow, clicked_start_group: clickedStartGroup, created_new: true };
    }
    finalHref = hrefNow || finalHref;
  }
  return {
    ok: false,
    reason: clickedStartGroup ? "new_group_chat_not_detected" : "start_group_chat_not_clicked",
    href: finalHref,
    clicked_start_group: clickedStartGroup,
  };
})()`,
        awaitPromise: true,
        returnByValue: true,
      });

      await sleep(1200);
      const current = await cdp.send("Runtime.evaluate", { expression: "location.href", returnByValue: true });
      let groupUrl = String(current?.result?.value ?? "");
      const value = step?.result?.value ?? {};
      if ((!groupUrl || groupUrl === "https://chatgpt.com/" || groupUrl === "https://chatgpt.com") && typeof value.href === "string") {
        groupUrl = value.href;
      }

      if (!value?.created_new || !groupUrl.startsWith("https://chatgpt.com/") || groupUrl === "https://chatgpt.com/" || groupUrl === "https://chatgpt.com") {
        errorResponse(res, 502, "group chat creation flow did not complete", { code: "group_chat_failed", details: value });
        return;
      }

      jsonResponse(res, 200, { ok: true, action: "create_group_chat", group_chat_url: groupUrl, details: value });
    } catch (err) {
      errorResponse(res, 502, err.message, { code: "group_chat_failed" });
    } finally {
      cdp.close();
    }
  };
}
