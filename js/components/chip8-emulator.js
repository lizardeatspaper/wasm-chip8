import ('../../crate/pkg/wasm_chip8_bg.wasm').then(({ memory }) => {
  import('../../crate/pkg').then(({ Emulator }) => {

    const WIDTH = 64
    const HEIGHT = 32
    const SCALE = 10
    const EMPTY_COLOR = '#0A84A0'
    const FILL_COLOR = '#ffffff'

    const getIndex = (row, col) => row * WIDTH + col

    customElements.define('chip8-emulator',
      class extends HTMLElement {
        constructor() {
          super()

          const shadow = this.attachShadow({ mode: 'open' })
          const $container = document.createElement('div')
          $container.classList.add('chip8-emulator')

          $container.innerHTML = `
        <style>
          .ch8e-canvas {
              background-color: ${EMPTY_COLOR};
              margin-top: 14px;
              margin-bottom: 14px;
          }
          
          .ch8e-upload-input {
              display: none;
          }
        </style>
        <div class="ch8e-controls">
            <upload-button>Upload game</upload-button>
            <button class="ch8e-start-btn">${this.started ? 'Pause' : 'Start'}</button>
        </div>
        <canvas class="ch8e-canvas" />
      `

          shadow.appendChild($container)

          this._programLoaded = false
          this._animationId = null
          this._emulator = Emulator.new()

          this.$canvas = $container.querySelector('canvas')
          this.$startBtn = $container.querySelector('button.ch8e-start-btn')
          this.$uploadBtn = $container.querySelector('upload-button')

          this._ctx = this.$canvas.getContext('2d')

          this.$canvas.height = (SCALE + 1) * HEIGHT + 1
          this.$canvas.width = (SCALE + 1) * WIDTH + 1

          this.renderGfx = this.renderGfx.bind(this)
          this.renderEmptyCells = this.renderEmptyCells.bind(this)
          this.renderFilledCells = this.renderFilledCells.bind(this)
          this.renderCellsByCond = this.renderCellsByCond.bind(this)
          this.loop = this.loop.bind(this)
          this.start = this.start.bind(this)
          this.pause = this.pause.bind(this)
          this.toggle = this.toggle.bind(this)
          this.startUpload = this.startUpload.bind(this)
          this.uploadProgram = this.uploadProgram.bind(this)
        }

        get started() {
          return this.getAttribute('started') === 'true' || false
        }

        set started(value) {
          this.setAttribute('started', value)
        }

        get ticksPerFrame() {
          return parseInt(this.getAttribute('ticks-per-frame'), 10) || 10
        }

        set ticksPerFrame(value) {
          this.setAttribute('ticks-per-frame', value)
        }

        connectedCallback() {
          this.$startBtn.addEventListener('click', this.toggle)
          this.$uploadBtn.addEventListener('file-selected', this.uploadProgram)
        }

        disconnectedCallback() {
          this.$startBtn.removeEventListener('click', this.toggle)
          this.$uploadBtn.removeEventListener('file-selected', this.uploadProgram)
        }

        renderGfx() {
          const gfx = new Uint8Array(memory.buffer, this._emulator.gfx(), WIDTH * HEIGHT)
          this._ctx.beginPath()
          this.renderFilledCells(gfx)
          this.renderEmptyCells(gfx)
          this._ctx.stroke()
        }

        renderCellsByCond(gfx, fillStyle, conditionCallback) {
          this._ctx.fillStyle = fillStyle
          for (let row = 0; row < HEIGHT; row++) {
            for (let col = 0; col < WIDTH; col++) {
              if (conditionCallback(row, col)) {
                continue
              }
              this._ctx.fillRect(
                col * (SCALE + 1) + 1,
                row * (SCALE + 1) + 1,
                SCALE,
                SCALE
              )
            }
          }
        }

        renderFilledCells(gfx) {
          this.renderCellsByCond(gfx, FILL_COLOR, (row, col) => !gfx[getIndex(row, col)])
        }

        renderEmptyCells(gfx) {
          this.renderCellsByCond(gfx, EMPTY_COLOR, (row, col) => gfx[getIndex(row, col)])
        }

        loop() {
          for (let i = 0; i < this.ticksPerFrame; i++) {
            this._emulator.tick()
          }
          this.renderGfx()
          this._animationId = requestAnimationFrame(this.loop)
        }

        start() {
          if (!this.started || this._programLoaded) {
            this.loop()
            this.started = true
          }
        }

        pause() {
          if (this.started) {
            cancelAnimationFrame(this._animationId)
            this._animationId = null
            this.started = false
          }
        }

        toggle() {
          this.started ? this.pause() : this.start()
        }

        startUpload() {
          this.$uploadInput.click()
        }

        uploadProgram(evt) {
          this._emulator.reset()
          this._ctx.clearRect(0, 0, this.$canvas.width, this.$canvas.height)
          this.pause()

          const self = this
          const file = evt.detail
          const reader = new FileReader()

          reader.onload = function (e) {
            const program = new Uint8Array(e.target.result)
            self._emulator.load(program)
            self._programLoaded = true
          }

          reader.readAsArrayBuffer(file)
        }
      }
    )
  })
})
