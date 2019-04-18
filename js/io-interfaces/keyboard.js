const KEYS_MAP = {
  49: 1, // 1
  50: 2, // 2
  51: 3, // 3
  81: 4, // Q
  87: 5, // W
  69: 6, // E
  65: 7, // A
  83: 8, // S
  68: 9, // D
  90: 10, // Z
  67: 11, // C
  52: 12, // 4
  82: 13, // R
  70: 14, // F
  86: 15 // V
}

export class Keyboard {
  constructor() {
    this.handle_keydown = this.handle_keydown.bind(this)
    this.handle_keyup = this.handle_keyup.bind(this)

    this.start_detection()
  }

  start_detection() {
    this.pressed = {}
    window.document.addEventListener('keydown', this.handle_keydown)
    window.document.addEventListener('keyup', this.handle_keyup)
  }

  handle_keydown(e) {
    this.pressed[KEYS_MAP[e.keyCode]] = true
  }

  handle_keyup(e) {
    this.pressed[KEYS_MAP[e.keyCode]] = false
  }

  is_key_pressed(key) {
    return Boolean(this.pressed[key])
  }
}
