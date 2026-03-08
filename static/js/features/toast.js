let toastContainer = null;

function createContainer() {
  const container = document.createElement('div');
  container.id = 'toast-container';
  container.style.cssText =
    'position:fixed;bottom:1rem;right:1rem;z-index:9999;display:flex;flex-direction:column;gap:0.5rem;pointer-events:none;';
  document.body.appendChild(container);
  return container;
}

function showToast(message, type) {
  if (!toastContainer) {
    toastContainer = document.getElementById('toast-container') || createContainer();
  }

  const toast = document.createElement('div');
  const bgColor = type === 'success' ? '#10b981' : type === 'error' ? '#ef4444' : '#6b7280';
  const icon = type === 'success' ? '✓' : type === 'error' ? '✕' : 'ℹ';
  toast.style.cssText = `background:${bgColor};color:white;padding:0.75rem 1rem;border-radius:0.5rem;box-shadow:0 4px 12px rgba(0,0,0,0.3);display:flex;align-items:center;gap:0.5rem;font-size:0.875rem;font-weight:500;animation:slideIn 0.3s ease-out;pointer-events:auto;`;
  toast.innerHTML = `<span style="font-size:1rem;">${icon}</span><span>${message}</span>`;

  toastContainer.appendChild(toast);

  setTimeout(() => {
    toast.style.animation = 'slideOut 0.3s ease-out';
    setTimeout(() => {
      if (toast.parentNode) toast.parentNode.removeChild(toast);
    }, 300);
  }, 3000);
}

function handleHtmxAfterSwap(event) {
  const trigger = event.detail.xhr.getResponseHeader('HX-Trigger');
  if (!trigger) return;

  const alreadyProcessed = event.detail.xhr._toastProcessed;
  if (alreadyProcessed) return;
  event.detail.xhr._toastProcessed = true;

  try {
    const data = JSON.parse(trigger);
    if (data.toast) {
      showToast(data.toast.message, data.toast.type);
    }
  } catch {
    // HX-Trigger might be a plain string, ignore
  }
}

export function initToast() {
  window.showToast = showToast;
  document.addEventListener('htmx:afterSwap', handleHtmxAfterSwap);
}
