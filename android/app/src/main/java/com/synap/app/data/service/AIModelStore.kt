package com.synap.app.data.service

import android.content.Context
import java.util.UUID
import org.json.JSONArray
import org.json.JSONObject

data class AIModelRecord(
    val id: String = UUID.randomUUID().toString(),
    val providerId: String,
    val serviceName: String,
    val baseUrl: String,
    val apiKey: String,
    val modelId: String,
    val modelName: String,
    val modelType: String,
    val isPreset: Boolean = false,
    val presetIconRes: Int? = null,
)

class AIModelStore(context: Context) {
    private val prefs = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)

    fun list(): List<AIModelRecord> {
        val raw = prefs.getString(KEY_MODELS, null) ?: return emptyList()
        val array = runCatching { JSONArray(raw) }.getOrNull() ?: return emptyList()
        return buildList {
            for (index in 0 until array.length()) {
                val item = array.optJSONObject(index) ?: continue
                add(item.toAIModelRecord())
            }
        }
    }

    fun add(record: AIModelRecord) {
        val current = list().toMutableList()
        current.add(record)
        save(current)
    }

    fun delete(id: String) {
        val current = list().toMutableList()
        current.removeAll { it.id == id }
        save(current)
    }

    private fun save(records: List<AIModelRecord>) {
        val payload = JSONArray().apply {
            records.forEach { record -> put(record.toJson()) }
        }
        prefs.edit().putString(KEY_MODELS, payload.toString()).apply()
    }

    private fun JSONObject.toAIModelRecord(): AIModelRecord = AIModelRecord(
        id = optString("id", UUID.randomUUID().toString()),
        providerId = optString("providerId"),
        serviceName = optString("serviceName"),
        baseUrl = optString("baseUrl"),
        apiKey = optString("apiKey"),
        modelId = optString("modelId"),
        modelName = optString("modelName"),
        modelType = optString("modelType", "LLM"),
        isPreset = optBoolean("isPreset", false),
        presetIconRes = if (has("presetIconRes")) optInt("presetIconRes") else null,
    )

    private fun AIModelRecord.toJson(): JSONObject = JSONObject().apply {
        put("id", id)
        put("providerId", providerId)
        put("serviceName", serviceName)
        put("baseUrl", baseUrl)
        put("apiKey", apiKey)
        put("modelId", modelId)
        put("modelName", modelName)
        put("modelType", modelType)
        put("isPreset", isPreset)
        presetIconRes?.let { put("presetIconRes", it) }
    }

    companion object {
        private const val PREFS_NAME = "ai_models"
        private const val KEY_MODELS = "models"
    }
}
