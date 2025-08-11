# Problem
So I have an ESP32-C3-supermini and want to send some wifi-packages with it (I am somehow really interested in the IEEE 802.11 standard.
However what I do not like is that the wifi driver of the ESP32-series are closed-sourced (at least the low-level stuff).
Espressif (the manufacturer of the soft- and hardware) made it so that you can only transmit beacon/probe request/probe response/action and non-QoS data frame. [see here](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv417esp_wifi_80211_tx16wifi_interface_tPKvib)
Now this limitation is set in the precompiled binary.

# "Solution"
There's a function "ieee80211_raw_frame_sanity_check" in the "libnet80211.a" > "ieee80211_output.o" file which returns 0 if the frame is valid.
<img width="1328" height="323" alt="Screenshot 2025-08-11 at 21 36 08" src="https://github.com/user-attachments/assets/eb6f0f7c-b33c-4f80-82e5-eb0544441857" />
In the image you can see the path were it would print that the frame-type is invalid (e.g. when I'm sending a Deauth frame) and later on return something else.
In the file "wifi_patch_and_run.fish" it searches for that particular place and returns 0 there. Now I do not get an error message when sending deauth frames.


The problem: I can still somehow **not** send Deauth frames. Any help would be appreciated.

Other people's work:
- https://github.com/risinek/esp32-wifi-penetration-tool
- https://github.com/GANESH-ICMC/esp32-deauther
- https://github.com/Jeija/esp32-80211-tx
- https://github.com/SpacehuhnTech/esp8266_deauther
- maybe more

These all use the idf development kit (asaik) with C. I want to use Rust though.
