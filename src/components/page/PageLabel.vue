<script setup lang="ts">
import { useConfirmModal } from '@/hooks/useConfirmModal'
import type { GridList } from '@/hooks/useGrid'
import { db } from '@/modules/database'
import type { InsertLabel, SelectLabel, SelectPlan } from '@/modules/database'

type LabelForm = Pick<InsertLabel, 'name' | 'planId' | 'color'>

interface SelectLabelDimension {
  labelId: number
  dimensionId: number
  name: string
  color: string
}

const timeStore = useTimerStore()
const { t } = useI18n()
const router = useRouter()
const { parseFieldsError } = useDatabase()
const { success } = useNotify()
const noteCreate = useNoteCreate()
const { onRefresh } = usePageRefresh()
const confirm = useConfirmModal()

const labelList = ref<GridList<SelectLabel>>([])
const { wrap, getItemsByOrder, select, selectedList } = useGrid(labelList)

const { running: timerRunning } = storeToRefs(timeStore)

const planList = ref<Array<SelectPlan>>([])
const dimensionList = ref<Array<SelectLabelDimension>>([])
const isCreate = ref(true)
const [listModeVisible, toggleListModeVisible] = useToggle(true)

const labelGroup = computed(() => {
  const map = new Map<number, GridList<SelectLabel>>()
  for (const label of labelList.value) {
    const list = map.get(label.planId)
    map.set(label.planId, [...(list || []), label])
  }
  const planIdList = planList.value.map(i => i.id)
  return [...map.entries()].sort((a, b) => planIdList.indexOf(a[0]) - planIdList.indexOf(b[0]))
})

const { setUpdateId, handleUpdate } = buildUpdateFn()

const { open, close, setModelValue } = useFormModal<LabelForm>(
  () => ({
    attrs: {
      title: isCreate.value ? t('label.create') : t('label.update'),
      form: {
        fields: [
          {
            type: 'textField',
            key: 'name',
            label: t('label.name'),
          },
          {
            type: 'autocomplete',
            key: 'planId',
            label: t('label.plan'),
            props: {
              items: planList.value.map(({ name, id }) => ({
                title: name,
                value: id,
              })),
              autoSelectFirst: 'exact',
            },
          },
          {
            type: 'colorPicker',
            key: 'color',
            label: t('label.color'),
          },
        ],
      },
      schema: z => z.object({
        name: z.string().min(1),
        planId: z.number(),
        color: z.string().length(7),
      }),
      async onConfirm(v, setErrors) {
        try {
          isCreate.value ? await handleCreate(v) : await handleUpdate(v)
        }
        catch (error) {
          return setErrors(parseFieldsError(error))
        }
        close()
        success({})
        refresh()
      },
    },
  }))

function openBatchRemoveModal() {
  confirm.delete({
    onConfirm: async () => {
      await db.label.batchRemove(selectedList.value)
      success({})
      refresh()
    },
  })
}

function handleRemove(id: number) {
  confirm.delete({
    onConfirm: async () => {
      await db.label.removeRelation(id)
      success({})
      refresh()
    },
  })
}

async function refresh() {
  [labelList.value, planList.value, dimensionList.value] = await Promise.all([
    db.label.select({ hidden: !listModeVisible.value }).then(wrap),
    db.plan.select(),
    db.label.selectDimension(),
  ])
}

function showCreateForm() {
  isCreate.value = true
  setModelValue({
    color: randomColor(),
  })
  open()
}

function showUpdateForm(id: number, list: GridList<SelectLabel>) {
  setUpdateId(id)
  const label = list.find(i => i.id == id)
  if (!label)
    return

  isCreate.value = false
  setModelValue(label)
  open()
}

async function handleCreate(label: LabelForm) {
  await db.label.transactionInsert(label)
}

function buildUpdateFn() {
  let id = 0
  return {
    setUpdateId: (updateId: number) => {
      id = updateId
    },
    handleUpdate: (label: LabelForm) => {
      return db.label.update(id, label)
    },
  }
}

async function handleStart(label: Pick<SelectLabel, 'id' | 'planId'>) {
  noteCreate.setModelValue({
    labelId: label.id,
    planId: label.planId,
  })
  let needJump = true
  try {
    const form = await noteCreate.open()
    if (form.direct)
      needJump = false
  }
  catch (error) {
    return
  }
  if (needJump)
    router.push('/timer')
  else
    refresh()
}

async function handleLayoutUpdated(items: number[], list: GridList<SelectLabel>) {
  const labelList = items.map((id, index) => {
    const { sort } = list[index]
    return {
      id,
      sort,
    }
  }).filter((i, index) => list[index].id != i.id)
  if (labelList.length) {
    await db.label.batchUpdate(labelList)
    await refresh()
  }
}

function getCardList(list: GridList<SelectLabel>) {
  return list.map(({ id, name, totalTime, color, planId, selected, hidden }) => ({
    id,
    title: name,
    totalTime,
    color,
    planId,
    selected,
    hidden,
  }))
}

function navigate(id: number) {
  router.push({
    path: '/timeline',
    query: {
      category: 'label',
      id,
    },
  })
}

async function toggleListMode() {
  toggleListModeVisible()
  await refresh()
}

function flip(value: number) {
  return value === 0 ? 1 : 0
}

async function handleToggleVisible(label: Pick<SelectLabel, 'id' | 'hidden'>) {
  await db.label.update(label.id, {
    hidden: flip(label.hidden),
  })
  await refresh()
  success({})
}

async function setBatchHidden() {
  const list = selectedList.value.map(id => ({ id, hidden: listModeVisible.value ? 1 : 0 }))
  await db.label.batchUpdate(list)
  await refresh()
  success({})
}

onRefresh(refresh)

refresh()
</script>

<template>
  <div v-if="labelList.length" py-4 space-y-6>
    <div v-for="[id, list] in labelGroup" :key="id">
      <template v-if="list.length">
        <div p-x-4 text-5 font-bold>
          {{ planList.find(i => i.id == id)!.name }}
        </div>
        <grid
          :items="getItemsByOrder(list)" :component-props="getCardList(list)" :options="{ cellHeight: 100 }"
          @layout-updated="items => handleLayoutUpdated(items, list)"
        >
          <template #default="{ componentProps }">
            <time-card
              v-bind="componentProps" @update="id => showUpdateForm(id, list)" @remove="handleRemove"
              @update:selected="v => select(componentProps.id, v)" @click="navigate(componentProps.id)"
            >
              <template v-if="!timerRunning" #menu>
                <v-list-item
                  value="timer" :title="$t('label.button.start')" append-icon="mdi-timer-outline"
                  @click="handleStart(componentProps)"
                />
                <v-list-item
                  value="listMode"
                  :title="Boolean(componentProps.hidden) ? $t('label.button.show') : $t('label.button.hide')"
                  :append-icon="Boolean(componentProps.hidden) ? 'i-mdi:eye' : 'i-mdi:eye-off'"
                  @click="handleToggleVisible(componentProps)"
                />
              </template>
              <template v-if="dimensionList.filter(i => i.labelId == componentProps.id).length > 0" #dimension>
                <v-chip
                  v-for="item in dimensionList.filter(i => i.labelId == componentProps.id)"
                  :key="item.dimensionId" :color="item.color" size="x-small" class="mr-1"
                >
                  {{ item.name }}
                </v-chip>
              </template>
            </time-card>
          </template>
        </grid>
      </template>
    </div>
  </div>
  <empty v-else type="label" :desc="$t('hint.label')" :width="250" />
  <more-menu>
    <v-list>
      <v-list-item
        v-if="selectedList.length" value="button.remove" :title="$t('button.remove')"
        append-icon="mdi-trash-can-outline" base-color="red" @click="openBatchRemoveModal"
      />
      <v-list-item value="label.create" :title="$t('label.create')" append-icon="mdi-plus" @click="showCreateForm" />
      <v-list-item
        v-if="selectedList.length" value="button.listMode" :title="listModeVisible ? $t('label.button.hide') : $t('label.button.show')"
        :append-icon="listModeVisible ? 'i-mdi:eye-off' : 'i-mdi:eye'" @click="setBatchHidden"
      />
    </v-list>
  </more-menu>
  <status-bar-teleport :xs="false">
    <status-bar-button
      :tooltip="listModeVisible ? $t('statusBar.listMode.tooltip.hide') : $t('statusBar.listMode.tooltip.show')"
      :icon="listModeVisible ? 'i-mdi:eye' : 'i-mdi:eye-off'" :text="$t('statusBar.listMode.text')"
      @click="toggleListMode"
    />
  </status-bar-teleport>
</template>
