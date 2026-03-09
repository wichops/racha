const SWIPE_THRESHOLD = 80;
const SWIPE_VELOCITY_THRESHOLD = 0.5;

let activeWrapper = null;
let activeCard = null;
let startX = 0;
let startY = 0;
let currentX = 0;
let startTime = 0;
let isDragging = false;
let isHorizontalSwipe = null;

function getTaskId(wrapper) {
  return wrapper.dataset.taskId;
}

function isTaskCompleted(wrapper) {
  return wrapper.dataset.completed === 'true';
}

function getCardFromWrapper(wrapper) {
  return wrapper.querySelector('.task-card');
}

function fireToggle(taskId) {
  htmx.ajax('POST', `/tasks/${taskId}/toggle`, {
    target: `#task-${taskId}`,
    swap: 'outerHTML',
  });
}

function handleTouchStart(event) {
  const wrapper = event.target.closest('.task-card-wrapper');
  if (!wrapper) return;

  const touch = event.touches[0];
  startX = touch.clientX;
  startY = touch.clientY;
  currentX = startX;
  startTime = Date.now();
  isDragging = true;
  isHorizontalSwipe = null;
  activeWrapper = wrapper;
  activeCard = getCardFromWrapper(wrapper);

  if (activeCard) {
    activeCard.style.transition = 'none';
  }
}

function handleTouchMove(event) {
  if (!isDragging || !activeWrapper || !activeCard) return;

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
}

function handleTouchEnd(_event) {
  if (!isDragging || !activeWrapper || !activeCard) return;

  const deltaX = currentX - startX;
  const deltaTime = Date.now() - startTime;
  const velocity = Math.abs(deltaX) / deltaTime;
  const completed = isTaskCompleted(activeWrapper);
  const taskId = getTaskId(activeWrapper);

  const crossedThreshold =
    Math.abs(deltaX) > SWIPE_THRESHOLD ||
    (velocity > SWIPE_VELOCITY_THRESHOLD && Math.abs(deltaX) > 40);

  if (crossedThreshold && isHorizontalSwipe) {
    if (deltaX < 0) {
      if (completed) {
        activeCard.style.transition = 'transform 0.2s ease';
        activeCard.style.transform = 'translateX(-40px)';
        setTimeout(() => fireToggle(taskId), 100);
      } else {
        activeCard.style.transition = 'transform 0.3s ease';
        activeCard.style.transform = 'translateX(0)';
      }
    } else {
      if (!completed) {
        activeCard.style.transition = 'transform 0.2s ease';
        activeCard.style.transform = 'translateX(40px)';
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
    }
    isDragging = false;
    activeWrapper = null;
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
      }
      isDragging = false;
      activeWrapper = null;
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
    const wrapper = event.target.closest('.task-card-wrapper');
    if (!wrapper) return;
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
