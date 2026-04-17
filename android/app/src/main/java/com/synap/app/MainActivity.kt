package com.synap.app

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.FileProvider
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import com.synap.app.data.service.SynapServiceApi
import com.synap.app.ui.viewmodel.HomeViewModel
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity : AppCompatActivity() {
    @Inject
    lateinit var synapService: SynapServiceApi

    private val homeViewModel: HomeViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        val splashScreen = installSplashScreen()
        super.onCreate(savedInstanceState)

        val startTime = System.currentTimeMillis()
        val timeout = 100L

        splashScreen.setKeepOnScreenCondition {
            val uiState = homeViewModel.uiState.value
            val currentTime = System.currentTimeMillis()
            uiState.isLoading && uiState.errorMessage == null && (currentTime - startTime < timeout)
        }

        // ========== 新增：处理冷启动时的选词分享 ==========
        handleProcessTextIntent(intent)

        setContent {
            SynapApp(activity = this)
        }
    }

    override fun onNewIntent(intent: Intent) {
        // ========== 新增：处理热启动（App在后台）时的选词分享 ==========
        handleProcessTextIntent(intent)
        setIntent(intent)
        super.onNewIntent(intent)
    }

    // ========== 核心魔改逻辑：拦截文本，转化为内部 DeepLink ==========
    private fun handleProcessTextIntent(intent: Intent?) {
        if (intent?.action == Intent.ACTION_PROCESS_TEXT) {
            val text = intent.getCharSequenceExtra(Intent.EXTRA_PROCESS_TEXT)?.toString()
            if (!text.isNullOrBlank()) {
                // 强制篡改 Intent，让 Compose Navigation 误以为这是一个 DeepLink 跳转
                intent.action = Intent.ACTION_VIEW
                intent.data = Uri.parse("synap://editor?initialContent=${Uri.encode(text)}")
            }
        }
    }

    suspend fun exportDatabaseToUri(uri: Uri): Result<Unit> = withContext(Dispatchers.IO) {
        runCatching {
            contentResolver.openOutputStream(uri)?.use { output ->
                synapService.exportDatabase(output).getOrThrow()
            } ?: error("无法创建导出文件")
        }
    }

    suspend fun shareDatabase(): Result<Unit> = withContext(Dispatchers.IO) {
        runCatching {
            val cachePath = File(cacheDir, "exports").apply { mkdirs() }
            val exportFile = File(cachePath, "synap_database.redb")
            exportFile.outputStream().use { output ->
                synapService.exportDatabase(output).getOrThrow()
            }
            val authority = "$packageName.fileprovider"
            val uri = FileProvider.getUriForFile(this@MainActivity, authority, exportFile)
            val shareIntent = Intent(Intent.ACTION_SEND).apply {
                type = "application/octet-stream"
                putExtra(Intent.EXTRA_STREAM, uri)
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            }
            withContext(Dispatchers.Main) {
                startActivity(Intent.createChooser(shareIntent, "分享数据库"))
            }
        }
    }

    suspend fun importDatabaseFromUri(uri: Uri): Result<Unit> = withContext(Dispatchers.IO) {
        runCatching {
            contentResolver.openInputStream(uri)?.use { input ->
                synapService.replaceDatabase(input).getOrThrow()
            } ?: error("无法读取导入文件")
        }
    }

    fun closeForDatabaseRestart() {
        finishAffinity()
    }
}