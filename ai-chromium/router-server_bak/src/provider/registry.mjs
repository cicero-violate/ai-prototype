const adapters = [];

export function register(adapter) {
  adapters.push(adapter);
}

export function resolve(model, request) {
  const explicit = request?.browser?.provider;
  if (explicit) {
    const found = adapters.find((a) => a.provider === explicit);
    if (!found) throw new Error(`unknown provider: "${explicit}"`);
    return found;
  }
  const found = adapters.find((a) => a.matches(model, request));
  if (!found) throw new Error(`no provider adapter matches model="${model}"`);
  return found;
}

export function getAdapter(provider) {
  const found = adapters.find((a) => a.provider === provider);
  if (!found) throw new Error(`unknown provider: "${provider}"`);
  return found;
}

export function listAdapters() {
  return adapters.map((a) => ({
    provider: a.provider,
    capabilities: a.capabilities,
    providerUrl: a.providerUrl,
  }));
}
