<script setup lang="ts">
import { FaviconService } from '@/modules/favicon'

const props = defineProps<{
  visible: boolean
}>()

const { visible: visibleVModel } = useVModels(props)

const store = useConfigStore()

const { config, autostart } = storeToRefs(store)

const localeOptions = [
  {
    title: '简体中文',
    value: 'zh-CN',
  },
  {
    title: 'English (United States)',
    value: 'en-US',
  },
]
</script>

<template>
  <advanced-dialog v-model:visible="visibleVModel" :title="$t('titleBar.view.setting')">
    <v-card-text class="sm:max-h-[400px]" overflow-y-auto>
      <v-list lines="two">
        <v-list-subheader>{{ $t('config.header.appearance') }}</v-list-subheader>
        <v-list-item :title="$t('config.themeColor')">
          <template #append>
            <color-picker-button v-model="config.themeColor" />
          </template>
        </v-list-item>
        <v-divider />
        <v-list-subheader>{{ $t('config.header.general') }}</v-list-subheader>
        <v-list-item>
          <v-list-item-title>{{ $t('config.locale') }}</v-list-item-title>
          <template #append>
            <v-list-item-action>
              <v-select v-model="config.locale" :items="localeOptions" hide-details class="w-[200px]" />
            </v-list-item-action>
          </template>
        </v-list-item>
        <v-list-item>
          <v-list-item-title>{{ $t('config.checkUpdate') }}</v-list-item-title>
          <template #append>
            <v-list-item-action>
              <v-checkbox-btn v-model="config.checkUpdate" />
            </v-list-item-action>
          </template>
        </v-list-item>
        <v-list-item>
          <v-list-item-title>{{ $t('config.autostart') }}</v-list-item-title>
          <template #append>
            <v-list-item-action>
              <v-checkbox-btn v-model="autostart" />
            </v-list-item-action>
          </template>
        </v-list-item>
        <v-list-item>
          <v-list-item-title>{{ $t('config.launchVisible') }}</v-list-item-title>
          <template #append>
            <v-list-item-action>
              <v-checkbox-btn v-model="config.launchVisible" />
            </v-list-item-action>
          </template>
        </v-list-item>
        <v-divider />
        <v-list-subheader>{{ $t('config.header.behavior') }}</v-list-subheader>
        <v-list-item>
          <v-list-item-title>
            {{ $t('config.timelineMinMinute') }}
          </v-list-item-title>
          <v-list-item-subtitle>
            {{ $t('config.desc.timelineMinMinute') }}
          </v-list-item-subtitle>
          <v-slider
            v-model="config.timelineMinMinute" px-4 py-2 thumb-label hide-details :min="0" :max="10" :step="1"
            @touchmove.stop
          />
        </v-list-item>
        <v-list-item>
          <v-list-item-title>
            {{ $t('config.timelineGroupGapMinute') }}
          </v-list-item-title>
          <v-list-item-subtitle>
            {{ $t('config.desc.timelineGroupGapMinute') }}
          </v-list-item-subtitle>
          <v-slider
            v-model="config.timelineGroupGapMinute" px-4 py-2 thumb-label hide-details :min="5" :max="60"
            :step="5" @touchmove.stop
          />
        </v-list-item>
        <v-list-item>
          <v-list-item-title>
            {{ $t('config.watcherWhitelist') }}
          </v-list-item-title>
          <v-list-item-subtitle>
            {{ $t('config.desc.watcherWhitelist') }}
          </v-list-item-subtitle>
          <watcher-whitelist v-model="config.watcherWhitelist" px-1 />
        </v-list-item>
        <v-list-item>
          <v-list-item-title>
            {{ $t('config.faviconService') }}
          </v-list-item-title>
          <v-list-item-subtitle>
            {{ $t('config.desc.faviconService') }}
          </v-list-item-subtitle>
          <v-radio-group v-model="config.faviconService" hide-details>
            <v-radio label="Google" :value="FaviconService.Google" />
            <v-radio label="Icon Horse" :value="FaviconService.IconHorse" />
          </v-radio-group>
        </v-list-item>
      </v-list>
    </v-card-text>
  </advanced-dialog>
</template>
