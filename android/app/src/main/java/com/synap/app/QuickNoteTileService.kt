package com.synap.app.widget

import android.app.PendingIntent
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService

class QuickNoteTileService : TileService() {

    // 当控制中心面板被下拉展开时调用
    override fun onStartListening() {
        super.onStartListening()
        qsTile?.apply {
            state = Tile.STATE_INACTIVE
            updateTile()
        }
    }

    // 当用户点击控制中心的这个按钮时调用
    override fun onClick() {
        super.onClick()

        // 1. 构造纯净的 DeepLink Intent
        val intent = Intent(Intent.ACTION_VIEW, Uri.parse("synap://editor")).apply {
            setPackage(packageName) // 限定只唤醒当前的 App，完美兼容你的图标切换功能
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK
        }

        // 2. 兼容不同 Android 版本的后台启动安全限制
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            // Android 14 (API 34) 及以上：必须使用 PendingIntent
            val pendingIntent = PendingIntent.getActivity(
                this,
                0,
                intent,
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
            )
            startActivityAndCollapse(pendingIntent)
        } else {
            // Android 13 及以下：使用传统的启动方式
            @Suppress("DEPRECATION")
            startActivityAndCollapse(intent)
        }
    }
}