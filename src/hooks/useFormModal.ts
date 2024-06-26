import type { UseModalOptions } from 'vue-final-modal'
import { useModal } from 'vue-final-modal'
import mergeOptions from 'merge-options'
import type { ComponentProps } from 'vue-component-type-helpers'
import deepEqual from 'fast-deep-equal'

import FormModal from '@/components/modal/FormModal.vue'

class ModalPromise {
  #promise?: Promise<void>
  #resolve?: () => void
  #reject?: () => void

  async open() {
    this.#promise = new Promise((resolve, reject) => {
      this.#resolve = resolve
      this.#reject = reject
    })
    return this.#promise
  }

  resolve() {
    const resolve = this.#resolve

    this.#promise = undefined
    this.#resolve = undefined
    this.#reject = undefined

    resolve?.()
  }

  reject() {
    const reject = this.#reject

    this.#promise = undefined
    this.#resolve = undefined
    this.#reject = undefined

    reject?.()
  }
}

export function useFormModal<
  T, V = any, C extends ComponentProps<typeof FormModal<T>> = ComponentProps<typeof FormModal<T>>, O extends UseModalOptions<C> = UseModalOptions<C>,
>(source: (model: Readonly<Partial<T>>, modalValue: Readonly<Partial<V>>) => O, setModalValue?: () => Promise<V>) {
  const { toggleDialog } = useDialogStore()

  const model: Ref<Partial<T>> = ref({})
  const modalValue: Ref<Partial<V>> = ref({})

  const promise = new ModalPromise()

  const options = mergeOptions(
    {
      attrs: {
        options: {
          reset: false,
        },
      },
    },
    source(model.value, modalValue.value),
    {
      attrs: {
        onFormUpdate(v: Partial<T>) {
          model.value = v
        },
        onClosed() {
          model.value = {}
          setModelValue({})
        },
        onAfterConfirm() {
          promise.resolve()
        },
        onAfterCancel() {
          promise.reject()
        },
      },
    })

  const modal = useModal<C>({
    component: FormModal,
    ...options,
  })

  async function open() {
    if (setModalValue)
      modalValue.value = await setModalValue()

    await modal.open()
    await promise.open()
    return model.value
  }

  const unwatchSource = watchDeep(() => source(model.value, modalValue.value), (v) => {
    const values = modal.options.attrs?.form.values
    if (Object.keys(values || {}).length > 0) {
      for (const key in values) {
        if (!deepEqual(values[key], model.value[key]))
          return
      }
    }

    const newOptions = mergeOptions(
      {
        attrs: modal.options.attrs,
      },
      {
        attrs: v.attrs,
      },
    )

    newOptions.attrs.form.values = {}

    modal.patchOptions(newOptions)
  })

  const unwatchDialog = watch(() => modal.options.modelValue, v => toggleDialog(v))

  onScopeDispose(() => {
    toggleDialog(false)
    unwatchSource()
    unwatchDialog()
  })

  function setModelValue(values: Partial<T>) {
    const validKeys = (modal.options.attrs?.form.fields || []).map(({ key }) => key)
    const newOptions = mergeOptions(
      {
        attrs: modal.options.attrs,
      },
      {
        attrs: {
          form: {
            values: includeKeys(values, validKeys),
          },
        },
      },
    )
    if (Object.keys(values).length == 0)
      newOptions.attrs.form.values = {}

    modal.patchOptions(newOptions)
  }

  return {
    ...modal,
    open,
    model,
    setModelValue,
  }
}
