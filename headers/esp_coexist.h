// Copyright 2015-2018 Espressif Systems (Shanghai) PTE LTD
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#ifndef __ESP_COEXIST_H__
#define __ESP_COEXIST_H__

#include <stdbool.h>
#include "esp_err.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief coex prefer value
 */
typedef enum {
    ESP_COEX_PREFER_WIFI = 0,       /*!< Prefer to WiFi, WiFi will have more opportunity to use RF */
    ESP_COEX_PREFER_BT,             /*!< Prefer to bluetooth, bluetooth will have more opportunity to use RF */
    ESP_COEX_PREFER_BALANCE,        /*!< Do balance of WiFi and bluetooth */
    ESP_COEX_PREFER_NUM,            /*!< Prefer value numbers */
} esp_coex_prefer_t;

/**
 * @brief coex status type
 */
typedef enum {
    ESP_COEX_ST_TYPE_WIFI = 0,
    ESP_COEX_ST_TYPE_BLE,
    ESP_COEX_ST_TYPE_BT,
} esp_coex_status_type_t;

/**
 * @brief coex configuration structure
 */
typedef struct
{
    uint8_t wifi_st;
    uint8_t ble_st;
    uint8_t bt_st;
    /* Members above shall vary from coexistence status macro: ESP_COEX_[MODULE]_ST_[STATUS] */
    uint8_t wifi_prop;
    uint8_t ble_prop;
    uint8_t bt_prop;
    /* proportion shall vary from 0-100 and the summary of them shall equal to 100 */ 
} esp_coex_proportion_t;

/* coexistence status macro */
#define ESP_COEX_WIIF_ST_SCAN              0x01
#define ESP_COEX_WIFI_ST_CONNECTING        0x02
#define ESP_COEX_WIFI_ST_CONNECTED         0x04

#define ESP_COEX_BLE_ST_IDLE               0x00
#define ESP_COEX_BLE_ST_ADV                0x01
#define ESP_COEX_BLE_ST_SCAN               0x02
#define ESP_COEX_BLE_ST_CONNECTED          0x04
#define ESP_COEX_BLE_ST_MESH_CONFIG        0x08
#define ESP_COEX_BLE_ST_MESH_TRAFFIC       0x10
#define ESP_COEX_BLE_ST_MESH_STANDBY       0x20

#define ESP_COEX_BT_ST_IDLE                0x00
#define ESP_COEX_BT_ST_ISCAN               0x01
#define ESP_COEX_BT_ST_CONNECTED           0x04
#define ESP_COEX_BT_ST_SNIFF               0x08
#define ESP_COEX_BT_ST_A2DP_STREAMING      0x10
#define ESP_COEX_BT_ST_A2DP_PAUSED         0x20

/**
 * @brief Get software coexist version string
 *
 * @return : version string
 */
const char *esp_coex_version_get(void);

/**
 * @deprecated Use esp_coex_status_bit_set() and esp_coex_status_bit_clear() instead.
 *  Set coexist preference of performance
 *  For example, if prefer to bluetooth, then it will make A2DP(play audio via classic bt)
 *  more smooth while wifi is runnning something.
 *  If prefer to wifi, it will do similar things as prefer to bluetooth.
 *  Default, it prefer to balance.
 *
 *  @param prefer : the prefer enumeration value
 *  @return : ESP_OK - success, other - failed
 */
esp_err_t esp_coex_preference_set(esp_coex_prefer_t prefer);

/**
 * @brief Set coex schm status
 * @param type : WIFI/BLE/BT
 * @param status : WIFI/BLE/BT STATUS
 * @return : ESP_OK - success, other - failed
 */
esp_err_t esp_coex_status_bit_set(esp_coex_status_type_t type, uint32_t status);

/**
 * @brief Clear coex schm status
 * @param type : WIFI/BLE/BT
 * @param status : WIFI/BLE/BT STATUS
 * @return : ESP_OK - success, other - failed
 */
esp_err_t esp_coex_status_bit_clear(esp_coex_status_type_t type, uint32_t status);

/**
 * @brief Set specifical coexistence proportion for different coexistence combination
 *  The following combinations are not supported to set proportion
 *  1. WiFi module is at IDLE status
 *  2. BLE module is at IDLE status and BT module is at IDLE / InquiryScan / Sniff / Inquiry status
 *  3. BLE module is at MESH_CONFIG / MESH_STANDBY status
 *  4. BLE module is at MESH TRAFFIC / SCAN with no mesh status and BT module is at Inquiry status
 *  @param prop : pointer for configuration structure
 *  @return : ESP_OK - success, other - failed
 */
esp_err_t esp_coex_proportion_set(esp_coex_proportion_t *prop);

/**
 * @brief Get specifical coexistence proportion at different coexistence combination
 *  The following combinations are not supported to get proportion
 *  1. WiFi module is at IDLE status
 *  2. BLE module is at IDLE status and BT module is at IDLE / InquiryScan / Sniff / Inquiry status
 *  3. BLE module is at MESH_CONFIG / MESH_STANDBY status
 *  4. BLE module is at MESH TRAFFIC / SCAN with no mesh status and BT module is at Inquiry status
 *  @param prop : pointer for configuration structure
 *  @return : ESP_OK - success, other - failed
 */
esp_err_t esp_coex_proportion_get(esp_coex_proportion_t *prop);

#ifdef __cplusplus
}
#endif


#endif /* __ESP_COEXIST_H__ */
