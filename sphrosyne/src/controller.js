// @ts-check

/**
 * Draw a circle on the canvas
 * @param {CanvasRenderingContext2D} ctx
 */
function drawCircle(ctx, { x, y, r }, fill = false) {
  ctx.beginPath();
  ctx.arc(x, y, r, 0, 2 * Math.PI);
  if (fill) ctx.fill();
  else ctx.stroke();
}

/**
 * Check if a touch object intersects a given circle
 * @param {{ x: number; y: number; r: number; }} circle
 * @param {Touch} touch
 */
function intersects(circle, touch) {
  const d =
    Math.pow(circle.x - touch.clientX, 2) +
    Math.pow(circle.y - touch.clientY, 2);
  return d <= Math.pow(circle.r, 2);
}

/**
 * Map a value from one numeric range from another
 * @param {number} input
 * @param {number} inputStart
 * @param {number} inputEnd
 * @param {number} outputStart
 * @param {number} outputEnd
 */
function map(input, inputStart, inputEnd, outputStart, outputEnd) {
  console.log(input, inputStart, inputEnd, outputStart, outputEnd);
  const slope = (outputEnd - outputStart) / (inputEnd - inputStart);
  return Math.round(outputStart + slope * (input - inputStart));
}

function copyTouch({ clientX, clientY, identifier }) {
  return { clientX, clientY, identifier };
}

window.addEventListener("DOMContentLoaded", function () {
  const canvas = document.createElement("canvas");
  const ctx = canvas.getContext("2d");
  document.body.append(canvas);
  ctx.lineWidth *= 2;

  let width, height, joystickRing, joystick, n, e, w, s, buttons;

  function onResize() {
    width = canvas.width = innerWidth;
    height = canvas.height = innerHeight;

    joystickRing = {
      x: width / 4 + 10,
      y: height / 2,
      r: height / 2 - 10,
    };

    joystick = {
      x: joystickRing.x,
      y: joystickRing.y,
      r: joystickRing.r / 4,
    };

    w = {
      r: width / 8,
      y: height / 2,
      x: joystickRing.x + joystickRing.r + width / 8,
      color: "blue",
      mask: 0x4000,
    };

    e = {
      r: width / 8,
      y: height / 2,
      x: w.x + w.r + width / 8,
      color: "red",
      mask: 0x2000,
    };

    n = {
      r: width / 8,
      y: height / 4,
      x: w.x + w.r,
      color: "gold",
      mask: 0x8000,
    };

    s = {
      r: width / 8,
      y: height - height / 4,
      x: w.x + w.r,
      color: "green",
      mask: 0x1000,
    };

    buttons = [n, e, w, s];
    buttons.forEach((b) => (b.r /= 1.5));
  }

  onResize();
  window.addEventListener("resize", onResize);
  window.addEventListener("beforeunload", () => ws.close());

  const ongoingTouches = new Map();

  // @ts-ignore
  const url = document.getElementById("url").value;
  const ws = new WebSocket(url);

  const mainloop = () => {
    ctx.fillStyle = "black";
    ctx.fillRect(0, 0, width, height);

    ctx.strokeStyle = "grey";
    ctx.fillStyle = "darkgrey";
    drawCircle(ctx, joystickRing);

    let lx = 0;
    let ly = 0;

    if (joystickTouch === null) drawCircle(ctx, joystick, true);
    else {
      const { clientX, clientY } = ongoingTouches.get(joystickTouch);
      lx = map(
        clientX - joystick.x,
        -joystickRing.r,
        joystickRing.r,
        -(1 << 15) - 1,
        1 << 15
      );
      ly = map(
        joystick.y - clientY,
        -joystickRing.r,
        joystickRing.r,
        -(1 << 15) - 1,
        1 << 15
      );
      drawCircle(ctx, { x: clientX, y: clientY, r: joystick.r }, true);
    }

    let buttonMask = 0;
    for (const button of buttons) {
      const pressed = Array.from(ongoingTouches.values()).some((touch) =>
        intersects(button, touch)
      );
      ctx.fillStyle = ctx.strokeStyle = button.color;
      drawCircle(ctx, button, pressed);

      if (pressed) buttonMask |= button.mask;
    }

    ws.send(
      JSON.stringify({
        buttons: buttonMask,
        left_trigger: 0,
        right_trigger: 0,
        left_thumbstick: [lx, ly],
        right_thumbstick: [0, 0],
      })
    );

    requestAnimationFrame(mainloop);
  };

  let joystickTouch = null;
  canvas.addEventListener("touchstart", (evt) => {
    evt.preventDefault();
    const touches = evt.changedTouches;

    for (const touch of touches) {
      if (joystickTouch === null && intersects(joystickRing, touch))
        joystickTouch = touch.identifier;

      ongoingTouches.set(touch.identifier, copyTouch(touch));
    }
  });
  canvas.addEventListener("touchmove", (evt) => {
    evt.preventDefault();
    const touches = evt.changedTouches;

    for (const touch of touches) {
      if (
        touch.identifier === joystickTouch &&
        !intersects(joystickRing, touch)
      )
        continue;

      ongoingTouches.set(touch.identifier, copyTouch(touch));
    }
  });
  canvas.addEventListener("touchend", (evt) => {
    evt.preventDefault();
    const touches = evt.changedTouches;

    for (const touch of touches) {
      ongoingTouches.delete(touch.identifier);
      if (touch.identifier == joystickTouch) joystickTouch = null;
    }
  });

  ws.addEventListener("open", () => requestAnimationFrame(mainloop));
});
