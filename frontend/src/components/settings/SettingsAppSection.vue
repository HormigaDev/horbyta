<template>
  <q-card flat bordered>
    <q-card-section>
      <div class="text-h6">{{ $t('settings.app.title') }}</div>
      <div class="text-caption" style="color: var(--text-muted)">
        {{ $t('settings.app.subtitle') }}
      </div>
    </q-card-section>

    <q-separator />

    <q-card-section class="row">
      <q-item clickable class="col-12 col-md-6 column items-center justify-center" to="/tags">
        <q-icon name="sell" size="lg" />
        <span class="text-subtitle2">{{ $t('settings.app.tags') }}</span>
      </q-item>
      <q-item clickable class="col-12 col-md-6 column items-center justify-center" to="/categories">
        <q-icon name="extension" size="lg" />
        <span class="text-subtitle2">{{ $t('settings.app.categories') }}</span>
      </q-item>
    </q-card-section>
  </q-card>
</template>
