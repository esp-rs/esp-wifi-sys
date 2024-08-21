#include <stdint.h>

typedef int _lock_t;

#define SOC_COEX_HW_PTI 1

typedef uint32_t        TickType_t;
typedef uint32_t        UBaseType_t;
typedef int32_t         BaseType_t;

typedef void*           QueueHandle_t;

typedef void*           esp_netif_t;
typedef void*           esp_netif_inherent_config_t;

struct ets_timer
{
  struct timer_adpt *next;
  uint32_t expire;
  uint32_t period;
  void (*func)(void *priv);
  void *priv;
};

struct timeval {
	uint64_t		tv_sec;		/* seconds */
	uint32_t	tv_usec;	/* and microseconds */
};

#include "esp_private/wifi.h"
#include "esp_wpa.h"
#include "esp_phy_init.h"
#include "phy.h"
#include "esp_timer.h"
#include "esp_eap_client.h"

#if !defined(CONFIG_IDF_TARGET_ESP32S2)
#include "esp_bt.h"
#include "esp_coexist_internal.h"
#include "esp_coexist_adapter.h"
#endif

#if ( defined(CONFIG_IDF_TARGET_ESP32C6) || defined(CONFIG_IDF_TARGET_ESP32H2) )
#include "esp_coexist.h"
#include "esp_coex_i154.h"
#endif

#include "esp_now.h"
