const HOLD_MS = 1000;
let activeEl = null;
let timer = null;

function cancel() {
  if (activeEl) {
    activeEl.classList.remove('holding');
    activeEl.classList.remove('hold-complete');
    activeEl = null;
  }
  if (timer) {
    clearTimeout(timer);
    timer = null;
  }
}

function fireToggle(taskId) {
  htmx.ajax('POST', `/tasks/${taskId}/toggle`, {
    target: `#task-${taskId}`,
    swap: 'outerHTML',
  });
}

function handlePointerDown(event) {
  const el = event.target.closest('.task-toggle');
  if (!el) return;
  event.preventDefault();

  cancel();
  activeEl = el;
  el.classList.add('holding');

  timer = setTimeout(() => {
    el.classList.remove('holding');
    el.classList.add('hold-complete');
    const taskId = el.dataset.taskId;

    setTimeout(() => {
      activeEl = null;
      timer = null;
      fireToggle(taskId);
    }, 150);
  }, HOLD_MS);
}

function handleContextMenu(event) {
  if (event.target.closest('.task-toggle')) {
    event.preventDefault();
  }
}

export function initTaskToggle() {
  document.addEventListener('pointerdown', handlePointerDown);
  document.addEventListener('pointerup', cancel);
  document.addEventListener('pointercancel', cancel);
  document.addEventListener('contextmenu', handleContextMenu);
}
