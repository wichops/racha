const SWIPE_THRESHOLD = 80;
const SWIPE_VELOCITY_THRESHOLD = 0.5;

let activeCard = null;
let startX = 0;
let startY = 0;
let currentX = 0;
let startTime = 0;
let isDragging = false;
let isHorizontalSwipe = null;

function getTaskId(card) {
  return card.dataset.taskId;
}

function isTaskCompleted(card) {
  return card.dataset.completed === 'true';
}

function fireToggle(taskId) {
  htmx.ajax('POST', `/tasks/${taskId}/toggle`, {
    target: `#task-${taskId}`,
    swap: 'outerHTML',
  });
}

function createSwipeHint() {
  const hint = document.createElement('div');
  hint.className = 'swipe-hint';
  hint.innerHTML = `
    <div class="swipe-hint-left">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="20 6 9 17 4 12"/>
      </svg>
      <span>Complete</span>
    </div>
    <div class="swipe-hint-right">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M3 12h18"/>
      </svg>
      <span>Uncomplete</span>
    </div>
  `;
  return hint;
}

function updateSwipeHint(card, translateX) {
  const hint = card.querySelector('.swipe-hint');
  if (!hint) return;

  const leftHint = hint.querySelector('.swipe-hint-left');
  const rightHint = hint.querySelector('.swipe-hint-right');

  if (translateX > 0) {
    rightHint.style.opacity = Math.min(translateX / SWIPE_THRESHOLD, 1);
    rightHint.style.transform = `scale(${0.8 + Math.min(translateX / SWIPE_THRESHOLD, 1) * 0.2})`;
    leftHint.style.opacity = 0;
  } else if (translateX < 0) {
    leftHint.style.opacity = Math.min(Math.abs(translateX) / SWIPE_THRESHOLD, 1);
    leftHint.style.transform = `scale(${0.8 + Math.min(Math.abs(translateX) / SWIPE_THRESHOLD, 1) * 0.2})`;
    rightHint.style.opacity = 0;
  }
}

function handleTouchStart(event) {
  const card = event.target.closest('.task-card');
  if (!card) return;

  const touch = event.touches[0];
  startX = touch.clientX;
  startY = touch.clientY;
  currentX = startX;
  startTime = Date.now();
  isDragging = true;
  isHorizontalSwipe = null;
  activeCard = card;

  if (!card.querySelector('.swipe-hint')) {
    card.appendChild(createSwipeHint());
  }

  card.style.transition = 'none';
}

function handleTouchMove(event) {
  if (!isDragging || !activeCard) return;

  const touch = event.touches[0];
  currentX = touch.clientX;
  const currentY = touch.clientY;
  const deltaX = currentX - startX;
  const deltaY = currentY - startY;

  if (isHorizontalSwipe === null) {
    const absX = Math.abs(deltaX);
    const absY = Math.abs(deltaY);
    if (absX > 10 || absY > 10) {
      isHorizontalSwipe = absX > absY;
    }
  }

  if (!isHorizontalSwipe) return;

  event.preventDefault();

  const translateX = deltaX * 0.6;
  activeCard.style.transform = `translateX(${translateX}px)`;
  updateSwipeHint(activeCard, translateX);
}

function handleTouchEnd(_event) {
  if (!isDragging || !activeCard) return;

  const deltaX = currentX - startX;
  const deltaTime = Date.now() - startTime;
  const velocity = Math.abs(deltaX) / deltaTime;
  const completed = isTaskCompleted(activeCard);
  const taskId = getTaskId(activeCard);

  const crossedThreshold =
    Math.abs(deltaX) > SWIPE_THRESHOLD ||
    (velocity > SWIPE_VELOCITY_THRESHOLD && Math.abs(deltaX) > 40);

  if (crossedThreshold && isHorizontalSwipe) {
    if (deltaX < 0) {
      if (!completed) {
        activeCard.style.transition = 'transform 0.2s ease';
        activeCard.style.transform = 'translateX(-100%)';
        setTimeout(() => fireToggle(taskId), 100);
      } else {
        activeCard.style.transition = 'transform 0.3s ease';
        activeCard.style.transform = 'translateX(0)';
      }
    } else {
      if (completed) {
        activeCard.style.transition = 'transform 0.2s ease';
        activeCard.style.transform = 'translateX(100%)';
        setTimeout(() => fireToggle(taskId), 100);
      } else {
        activeCard.style.transition = 'transform 0.3s ease';
        activeCard.style.transform = 'translateX(0)';
      }
    }
  } else {
    activeCard.style.transition = 'transform 0.3s ease';
    activeCard.style.transform = 'translateX(0)';
  }

  setTimeout(() => {
    if (activeCard) {
      activeCard.style.transform = '';
      activeCard.style.transition = '';
      const hint = activeCard.querySelector('.swipe-hint');
      if (hint) hint.remove();
    }
    isDragging = false;
    activeCard = null;
    isHorizontalSwipe = null;
  }, 300);
}

function handleTouchCancel() {
  if (activeCard) {
    activeCard.style.transition = 'transform 0.3s ease';
    activeCard.style.transform = 'translateX(0)';
    setTimeout(() => {
      if (activeCard) {
        activeCard.style.transform = '';
        activeCard.style.transition = '';
        const hint = activeCard.querySelector('.swipe-hint');
        if (hint) hint.remove();
      }
      isDragging = false;
      activeCard = null;
      isHorizontalSwipe = null;
    }, 300);
  }
}

export function initTaskSwipe() {
  document.addEventListener('touchstart', handleTouchStart, { passive: true });
  document.addEventListener('touchmove', handleTouchMove, { passive: false });
  document.addEventListener('touchend', handleTouchEnd, { passive: true });
  document.addEventListener('touchcancel', handleTouchCancel, { passive: true });

  let mouseDown = false;

  document.addEventListener('mousedown', (event) => {
    const card = event.target.closest('.task-card');
    if (!card) return;
    mouseDown = true;
    handleTouchStart({
      touches: [{ clientX: event.clientX, clientY: event.clientY }],
      target: event.target,
    });
  });

  document.addEventListener('mousemove', (event) => {
    if (!mouseDown) return;
    handleTouchMove({
      touches: [{ clientX: event.clientX, clientY: event.clientY }],
      preventDefault: () => {},
      target: event.target,
    });
  });

  document.addEventListener('mouseup', (event) => {
    if (!mouseDown) return;
    mouseDown = false;
    handleTouchEnd({
      target: event.target,
    });
  });

  document.addEventListener('mouseleave', () => {
    if (mouseDown) {
      mouseDown = false;
      handleTouchCancel();
    }
  });
}
