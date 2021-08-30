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
      if (touch.identifier === this.touch)
        if (intersects(this.ring, touch)) {
          this.stickX = touch.clientX;
          this.stickY = touch.clientY;
        } else {
          // If the touch point does not land within our ring, project the point onto its edge and set that as the stick's position
          // Math courtesy of https://math.stackexchange.com/a/127615
          this.stickX =
            this.centerX +
            (this.outerRadius * (touch.clientX - this.centerX)) /
              Math.hypot(
                touch.clientX - this.centerX,
                touch.clientY - this.centerY
              );
          this.stickY =
            this.centerY +
            (this.outerRadius * (touch.clientY - this.centerY)) /
              Math.hypot(
                touch.clientX - this.centerX,
                touch.clientY - this.centerY
              );
        }
      break;
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
        this.stickY = this.centerY;
      }
    }
  }

  /**
   * @param {CanvasRenderingContext2D} ctx
   */
  draw(ctx) {
    ctx.strokeStyle = "grey";
    ctx.fillStyle = "darkgrey";
    drawCircle(ctx, this.ring);
    drawCircle(ctx, this.stick, true);
  }
}

class Buttons {
  /**
   * Construct a new set of buttons, which will be placed around the pivot circle
   * @param {{ x: number; y: number; r: number; }} pivot
   * @param {number} buttonRadius
   * @param {{ color: string; mask: number; } []} [extra]
   */
  constructor(pivot, buttonRadius, extra) {
    const north = {
      x: pivot.x,
      y: pivot.y - pivot.r - buttonRadius,
      r: buttonRadius,
    };

    const south = {
      x: pivot.x,
      y: pivot.y + pivot.r + buttonRadius,
      r: buttonRadius,
    };

    const west = {
      x: pivot.x - pivot.r - buttonRadius,
      y: pivot.y,
      r: buttonRadius,
    };

    const east = {
      x: pivot.x + pivot.r + buttonRadius,
      y: pivot.y,
      r: buttonRadius,
    };

    this.buttons = [north, east, west, south].map((r, i) =>
      Object.assign(r, extra[i])
    );

    this.state = 0;
  }

  /**
   * @param {CanvasRenderingContext2D} ctx
   * @param {Map<number, Touch>} ongoingTouches
   */
  draw(ctx, ongoingTouches) {
    this.state = 0;
    for (const button of this.buttons) {
      const pressed = Array.from(ongoingTouches.values()).some((touch) =>
        intersects(button, touch)
      );
      ctx.fillStyle = ctx.strokeStyle = button.color;
      drawCircle(ctx, button, pressed);

      if (pressed) this.state |= button.mask;
    }
  }
}

window.addEventListener("DOMContentLoaded", function () {
  const canvas = document.createElement("canvas");
  const ctx = canvas.getContext("2d");
  document.body.append(canvas);
  ctx.lineWidth *= 2;

  let leftJoystick, rightJoystick, leftButtons, rightButtons;

  function buildScene() {
    const width = (canvas.width = innerWidth);
    const height = (canvas.height = innerHeight);
    const buttonRadius = width / 24;
    const rightPivot = {
      x: width / 2 + width / 8,
      y: height / 4 + 10,
      r: buttonRadius / 2,
    };
    const leftPivot = {
      x: width / 2 - width / 8,
      y: height / 4 + 10,
      r: buttonRadius / 2,
    };
    leftJoystick = new Joystick(
      width / 4 + 10,
      height - height / 4,
      height / 4 - 10,
      4
    );
    rightJoystick = new Joystick(
      width - (width / 4 + 10),
      height - height / 4,
      height / 4 - 10,
      4
    );
    leftButtons = new Buttons(leftPivot, buttonRadius, [
      { color: "orange", mask: 0x0001 },
      { color: "orange", mask: 0x0008 },
      { color: "orange", mask: 0x0004 },
      { color: "orange", mask: 0x0002 },
    ]);
    rightButtons = new Buttons(rightPivot, buttonRadius, [
      { color: "gold", mask: 0x8000 },
      { color: "green", mask: 0x1000 },
      { color: "blue", mask: 0x4000 },
      { color: "red", mask: 0x2000 },
    ]);
  }

  buildScene();
  window.addEventListener("resize", buildScene);
  window.addEventListener("beforeunload", () => ws.close());

  const ongoingTouches = new Map();

  // @ts-ignore
  const url = document.getElementById("url").value;
  const ws = new WebSocket(url);

  function mainloop() {
    ctx.fillStyle = "black";
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    leftJoystick.draw(ctx);
    rightJoystick.draw(ctx);
    leftButtons.draw(ctx, ongoingTouches);
    rightButtons.draw(ctx, ongoingTouches);

    if (ws.readyState === ws.OPEN)
      ws.send(
        JSON.stringify({
          buttons: leftButtons.state | rightButtons.state,
          left_trigger: 0,
          right_trigger: 0,
          left_thumbstick: leftJoystick.stickValue,
          right_thumbstick: rightJoystick.stickValue,
        })
      );

    requestAnimationFrame(mainloop);
  }

  canvas.addEventListener("touchstart", (event) => {
    event.preventDefault();

    leftJoystick.ontouchstart(event);
    rightJoystick.ontouchstart(event);
    for (const touch of event.changedTouches) {
      ongoingTouches.set(touch.identifier, copyTouch(touch));
    }
  });
  canvas.addEventListener("touchmove", (event) => {
    event.preventDefault();

    leftJoystick.ontouchmove(event);
    rightJoystick.ontouchmove(event);
    for (const touch of event.changedTouches) {
      ongoingTouches.set(touch.identifier, copyTouch(touch));
    }
  });
  canvas.addEventListener("touchend", (event) => {
    event.preventDefault();

    leftJoystick.ontouchend(event);
    rightJoystick.ontouchend(event);
    for (const touch of event.changedTouches) {
      ongoingTouches.delete(touch.identifier);
    }
  });

  mainloop();
});
