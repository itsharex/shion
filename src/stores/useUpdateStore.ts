import type { Update } from '@tauri-apps/plugin-updater'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { error } from '@tauri-apps/plugin-log'

export const useUpdateStore = defineStore('update', () => {
  const { t } = useI18n()
  const notify = useNotify()
  const confirm = useConfirmModal()

  const configStore = useConfigStore()

  const { config } = storeToRefs(configStore)

  const updating = ref(false)
  const needUpdate = ref(false)

  async function getUpdate() {
    return await check({
      timeout: 6,
    })
  }

  async function start(showInfo = false) {
    updating.value = true
    let update: Update | null
    try {
      update = await getUpdate()
    }
    catch (e) {
      updating.value = false
      notify.error({
        text: t('updater.checkUpdate'),
      })
      return error(`update check error: ${e}`)
    }

    if (update) {
      needUpdate.value = true
      const openModal = () => {
        confirm.require({
          title: t('updater.title'),
          content: t('updater.content', {
            version: update!.version,
          }),
          options: {
            loading: true,
          },
          onConfirm: async () => {
            try {
              await update!.downloadAndInstall()
              await relaunch()
            }
            catch (e) {
              updating.value = false
              error(`update downloadAndInstall error: ${e}`)
              if ((e as string) == 'UnexpectedKeyId') {
                notify.info({
                  text: t('updater.reinstall'),
                })
              }
              else {
                notify.error({
                  text: t('updater.updating'),
                })
              }
            }
          },
          onClosed() {
            updating.value = false
          },
        })
      }

      return openModal
    }
    else {
      updating.value = false
      if (showInfo) {
        notify.info({
          text: t('updater.latest'),
        })
      }
    }
  }

  watchOnce(() => config.value.checkUpdate, async (v) => {
    if (v) {
      const open = await start()
      open?.()
    }
  })

  const _timer = new Timer(async () => {
    const update = await getUpdate()
    if (update)
      needUpdate.value = true
  }, calcDuration(6, 'hour'), true)

  return {
    start,
    updating,
    needUpdate,
  }
})
