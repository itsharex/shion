import type { ComponentProps } from 'vue-component-type-helpers'
import type { VAutocomplete, VCheckbox, VColorPicker, VSelect, VTextField } from 'vuetify/components'
import type { z as Zod, ZodObject } from 'zod'

import type NestedMenu from '../shared/NestedMenu.vue'
import type FilePicker from '../form/FilePicker.vue'
import type DatetimePicker from '../form/DatetimePicker.vue'

export interface FormItemProps {
  textField: ComponentProps<typeof VTextField>
  colorPicker: ComponentProps<typeof VColorPicker>
  select: ComponentProps<typeof VSelect>
  autocomplete: ComponentProps<typeof VAutocomplete>
  checkbox: ComponentProps<typeof VCheckbox>
  cascader: ComponentProps<typeof NestedMenu>
  filePicker: ComponentProps<typeof FilePicker>
  datetimePicker: ComponentProps<typeof DatetimePicker>
}

export interface FormItem<T extends keyof FormItemProps, K> {
  type: T
  key: K
  label: string
  visible?: boolean
  props?: FormItemProps[T]
}

export interface Form<K> {
  fields: Array<FormItem<keyof FormItemProps, K>>
  values?: Record<string, unknown>
}

export type BuildSchemaObject = (z: typeof Zod) => ZodObject<any, any, any, any, any>
