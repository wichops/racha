import { getLocalDate } from '../utils.js';

function handleHtmxConfigRequest(event) {
  event.detail.headers['X-Local-Date'] = getLocalDate();
}

export function initDateHeader() {
  document.addEventListener('htmx:configRequest', handleHtmxConfigRequest);
}
