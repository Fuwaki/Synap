package com.synap.app.widget

import android.app.PendingIntent
import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProvider
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.widget.RemoteViews
import com.synap.app.R

class QuickNoteWidget : AppWidgetProvider() {
    override fun onUpdate(context: Context, appWidgetManager: AppWidgetManager, appWidgetIds: IntArray) {
        for (appWidgetId in appWidgetIds) {
            val views = RemoteViews(context.packageName, R.layout.widget_quick_note)

            // 1. 动态获取当前激活的 Launcher 组件
            val launchIntent = context.packageManager.getLaunchIntentForPackage(context.packageName)
            val componentName = launchIntent?.component

            // 2. 构造意图
            val intent = Intent(Intent.ACTION_VIEW, Uri.parse("synap://editor")).apply {
                component = componentName
                // ========== 终极修复：使用 CLEAR_TASK ==========
                // NEW_TASK + CLEAR_TASK 的组合，会瞬间清空 App 之前的所有后台页面，
                // 完美模拟了“杀掉进程重新打开”的干净状态，每次点击 100% 触发全新启动的 DeepLink！
                flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK
            }

            // 3. 构造 PendingIntent
            val pendingIntent = PendingIntent.getActivity(
                context,
                appWidgetId,
                intent,
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
            )

            views.setOnClickPendingIntent(R.id.widget_btn_add, pendingIntent)
            appWidgetManager.updateAppWidget(appWidgetId, views)
        }
    }
}