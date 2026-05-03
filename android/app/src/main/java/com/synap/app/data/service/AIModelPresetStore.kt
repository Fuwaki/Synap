package com.synap.app.data.service

import android.content.Context
import android.util.Log
import org.json.JSONArray
import org.json.JSONObject

data class PresetModelProvider(
    val id: String,
    val name: String,
    val iconResName: String,
    val baseURL: String,
    val models: List<PresetModelItem>,
)

data class PresetModelItem(
    val id: String,
    val name: String,
)

object AIModelPresetStore {
    private const val TAG = "AIModelPresetStore"
    private var providers: List<PresetModelProvider>? = null

    fun load(context: Context): List<PresetModelProvider> {
        providers?.let { return it }

        return try {
            val json = context.assets.open("ai_preset_models.json").bufferedReader().use { it.readText() }
            val root = JSONObject(json)
            val providersArray = root.getJSONArray("providers")

            val result = mutableListOf<PresetModelProvider>()
            for (i in 0 until providersArray.length()) {
                val obj = providersArray.getJSONObject(i)
                val modelsArray = obj.getJSONArray("models")
                val models = mutableListOf<PresetModelItem>()
                for (j in 0 until modelsArray.length()) {
                    val modelObj = modelsArray.getJSONObject(j)
                    models.add(
                        PresetModelItem(
                            id = modelObj.getString("id"),
                            name = modelObj.getString("name"),
                        )
                    )
                }
                result.add(
                    PresetModelProvider(
                        id = obj.getString("id"),
                        name = obj.getString("name"),
                        iconResName = obj.getString("icon"),
                        baseURL = obj.getString("baseURL"),
                        models = models,
                    )
                )
            }

            providers = result
            result
        } catch (e: Exception) {
            Log.e(TAG, "Failed to load preset models", e)
            emptyList()
        }
    }

    fun getIconRes(iconResName: String): Int {
        return try {
            val clazz = Class.forName("com.synap.app.R\$drawable")
            val field = clazz.getField(iconResName)
            field.getInt(null)
        } catch (e: Exception) {
            Log.w(TAG, "Icon not found: $iconResName", e)
            0
        }
    }
}
