export class Audio {
  constructor() {
    this.ctx = new AudioContext()
    this.o = null
  }

  start() {
    if (!this.is_active()) {
      this.o = this.ctx.createOscillator()
      this.o.type = 'sine'
      this.o.connect(this.ctx.destination)
      this.o.start()
    }
  }

  stop() {
    if (this.is_active()) {
      this.o.stop()
      this.o = null
    }
  }

  is_active() {
    return Boolean(this.o)
  }
}
