import type { Update } from '@tauri-apps/plugin-updater'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { error } from '@tauri-apps/plugin-log'

export const useUpdateStore = defineStore('update', () => {
  const { t } = useI18n()
  const notify = useNotify()
  const configStore = useConfigStore()

  const { config } = storeToRefs(configStore)

  const updating = ref(false)

  async function start() {
    updating.value = true
    let update: Update | null
    try {
      update = await check({
        timeout: 6,
      })
    }
    catch (e) {
      updating.value = false
      notify.error({
        text: t('updater.checkUpdate'),
      })
      return error(e as string)
    }

    if (update?.version) {
      if (update?.version == config.value.version) {
        return notify.info({
          text: t('updater.latest'),
        })
      }
      const { open, close } = useConfirmModal({
        attrs: {
          title: t('updater.title'),
          content: t('updater.content', {
            version: update!.version,
          }),
          async onConfirm() {
            try {
              await update!.downloadAndInstall()
              await relaunch()
            }
            catch (e) {
              close()
              notify.error({
                text: t('updater.updating'),
              })
              error(e as string)
            }
          },
          onClosed() {
            updating.value = false
          },
        },
      })
      open()
    }
  }

  watchOnce(() => config.value.checkUpdate, (v) => {
    if (v)
      start()
  })

  return {
    start,
    updating,
  }
})
