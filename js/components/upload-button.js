customElements.define('upload-button',
  class extends HTMLElement {
    constructor() {
      super()

      const shadow = this.attachShadow({ mode: 'open' })
      const button = document.createElement('button')

      button.innerHTML = `
        <style scoped>
            .csub-upload-input {
                display: none;
            }
        </style>
        <input class="csub-upload-input" type="file">
        <slot>TEXT MISSING</slot>
      `

      shadow.appendChild(button)

      this.$file = shadow.querySelector('input[type="file"]')

      this.handleClick = this.handleClick.bind(this)
      this.handleFileSelected = this.handleFileSelected.bind(this)
    }

    connectedCallback() {
      this.addEventListener('click', this.handleClick)
      this.$file.addEventListener('change', this.handleFileSelected)
    }

    disconnectedCallback() {
      this.removeEventListener('click', this.handleClick)
      this.$file.removeEventListener('change', this.handleFileSelected)
    }

    handleClick() {
      this.$file.click()
    }

    handleFileSelected(evt) {
      this.dispatchEvent(new CustomEvent('file-selected', {
        bubbles: true,
        cancelable: true,
        detail: evt.target.files[0]
      }))
    }
  }
)
