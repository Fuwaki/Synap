package com.synap.app.data.search

import android.content.Context
import androidx.appsearch.app.AppSearchSession
import androidx.appsearch.app.PutDocumentsRequest
import androidx.appsearch.app.RemoveByDocumentIdRequest
import androidx.appsearch.app.SetSchemaRequest
import androidx.appsearch.platformstorage.PlatformStorage
import com.google.common.util.concurrent.ListenableFuture
import kotlinx.coroutines.suspendCancellableCoroutine
import java.util.concurrent.ExecutionException
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException

class SynapSearchManager(private val context: Context) {

    private suspend fun getSearchSession(): AppSearchSession {
        // 使用 PlatformStorage，这是让系统全局搜索读取到的关键！
        val sessionFuture = PlatformStorage.createSearchSessionAsync(
            PlatformStorage.SearchContext.Builder(context, "synap_search_db").build()
        )
        val session = sessionFuture.await()

        // 注册 Schema，并设置为对系统可见
        val setSchemaRequest = SetSchemaRequest.Builder()
            .addDocumentClasses(SearchableNote::class.java)
            // 修复 API 名称：允许系统 UI 读取这个 Schema 的数据
            .setSchemaTypeDisplayedBySystem("SearchableNote", true)
            .build()

        session.setSchemaAsync(setSchemaRequest).await()
        return session
    }

    // 新增或更新笔记到索引
    suspend fun indexNote(noteId: String, content: String, timestamp: Long) {
        val session = getSearchSession()
        val searchableNote = SearchableNote(
            namespace = "notes",
            id = noteId,
            score = 100, // 可以根据逻辑动态算分
            content = content,
            creationTimestampMillis = timestamp
        )

        val putRequest = PutDocumentsRequest.Builder()
            .addDocuments(searchableNote)
            .build()

        session.putAsync(putRequest).await()
    }

    // 从索引中删除笔记
    suspend fun removeNote(noteId: String) {
        val session = getSearchSession()
        session.removeAsync(
            RemoveByDocumentIdRequest.Builder("notes")
                .addIds(noteId)
                .build()
        ).await()
    }
}

// ========== 免除依赖：手写 ListenableFuture 转协程的 await 扩展 ==========
suspend fun <T> ListenableFuture<T>.await(): T = suspendCancellableCoroutine { cont ->
    addListener(
        {
            try {
                cont.resume(get())
            } catch (e: ExecutionException) {
                cont.resumeWithException(e.cause ?: e)
            } catch (e: Exception) {
                cont.resumeWithException(e)
            }
        },
        { it.run() } // 直接在当前线程执行回调（Direct Executor）
    )
}