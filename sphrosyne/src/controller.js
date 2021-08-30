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
  const slope = (outputEnd - outputStart) / (inputEnd - inputStart);
  return Math.round(outputStart + slope * (input - inputStart));
}

function copyTouch({ clientX, clientY, identifier }) {
  return { clientX, clientY, identifier };
}

class Joystick {
  /**
   * @param {number} x
   * @param {number} y
   * @param {number} outerRadius
   * @param {number} innerRadiusDivisor
   */
  constructor(x, y, outerRadius, innerRadiusDivisor) {
    this.centerX = x;
    this.centerY = y;
    this.outerRadius = outerRadius;
    this.innerRadius = outerRadius / innerRadiusDivisor;
    this.touch = null;

    this.stickX = x;
    this.stickY = y;
  }

  get ring() {
    return { x: this.centerX, y: this.centerY, r: this.outerRadius };
  }

  get stick() {
    return { x: this.stickX, y: this.stickY, r: this.innerRadius };
  }

  get stickValue() {
    return [
      map(
        this.stickX - this.centerX,
        -this.outerRadius,
        this.outerRadius,
        -(1 << 15) - 1,
        1 << 15
      ),
      map(
        this.centerY - this.stickY,
        -this.outerRadius,
        this.outerRadius,
        -(1 << 15) - 1,
        1 << 15
      ),
    ];
  }

  /**
   * @param {TouchEvent} event
   */
  ontouchstart(event) {
    if (this.touch !== null) return;

    for (const touch of event.changedTouches) {
      if (intersects(this.ring, touch)) {
        this.touch = touch.identifier;
        this.stickX = touch.clientX;
        this.stickY = touch.clientY;
        break;
      }
    }
  }

  /**
   * @param {TouchEvent} event
   */
  ontouchmove(event) {
    if (this.touch === null) return;

    for (const touch of event.changedTouches) {
      if (touch.identifier === this.touch && intersects(this.ring, touch)) {
        this.stickX = touch.clientX;
        this.stickY = touch.clientY;
        break;
      }
    }
  }

  /**
   * @param {TouchEvent} event
   */
  ontouchend(event) {
    for (const touch of event.changedTouches) {
      if (touch.identifier === this.touch) {
        this.touch = null;
        this.stickX = this.centerX;
        this.stickY = this.centerX;
      }
    }
  }
}

window.addEventListener("DOMContentLoaded", function () {
  const canvas = document.createElement("canvas");
  const ctx = canvas.getContext("2d");
  document.body.append(canvas);
  ctx.lineWidth *= 2;

  let joystick, buttons;

  function onResize() {
    const width = (canvas.width = innerWidth);
    const height = (canvas.height = innerHeight);

    joystick = new Joystick(width / 4 + 10, height / 2, height / 2 - 10, 4);

    const west = {
      r: width / 8,
      y: height / 2,
      x: joystick.centerX + joystick.outerRadius + width / 8,
      color: "blue",
      mask: 0x4000,
    };

    const east = {
      r: width / 8,
      y: height / 2,
      x: west.x + west.r + width / 8,
      color: "red",
      mask: 0x2000,
    };

    const north = {
      r: width / 8,
      y: height / 4,
      x: west.x + west.r,
      color: "gold",
      mask: 0x8000,
    };

    const south = {
      r: width / 8,
      y: height - height / 4,
      x: west.x + west.r,
      color: "green",
      mask: 0x1000,
    };

    buttons = [north, east, west, south];
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
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    ctx.strokeStyle = "grey";
    ctx.fillStyle = "darkgrey";
    drawCircle(ctx, joystick.ring);

    drawCircle(ctx, joystick.stick, true);

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
        left_thumbstick: joystick.stickValue,
        right_thumbstick: [0, 0],
      })
    );

    requestAnimationFrame(mainloop);
  };

  let joystickTouch = null;
  canvas.addEventListener("touchstart", (event) => {
    event.preventDefault();

    joystick.ontouchstart(event);
    for (const touch of event.changedTouches) {
      ongoingTouches.set(touch.identifier, copyTouch(touch));
    }
  });
  canvas.addEventListener("touchmove", (event) => {
    event.preventDefault();

    joystick.ontouchmove(event);
    for (const touch of event.changedTouches) {
      ongoingTouches.set(touch.identifier, copyTouch(touch));
    }
  });
  canvas.addEventListener("touchend", (event) => {
    event.preventDefault();

    joystick.ontouchend(event);
    for (const touch of event.changedTouches) {
      ongoingTouches.delete(touch.identifier);
    }
  });

  ws.addEventListener("open", () => requestAnimationFrame(mainloop));
});
