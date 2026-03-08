import { initDateHeader } from './features/date-header.js';
import { initTaskSwipe } from './features/task-swipe.js';
import { initTaskToggle } from './features/task-toggle.js';
import { initToast } from './features/toast.js';
import { getLocalDate, setCookie } from './utils.js';

function init() {
  setCookie('local_date', getLocalDate());
  initDateHeader();
  initTaskToggle();
  initTaskSwipe();
  initToast();
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
