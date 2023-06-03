# watchcorgi

A simple multi-GPU server management tool.

![overview](./watchcorgi.drawio.png)

# Pre-preparation

Make sure the monitoring server has `redis-server` installed that without password.

# Installation

Download the `watchcorgi-client` and `watchcorgi-server` programs separately, put the `watchcorgi-client` on the GPU server you want to monitor, and the `watchcorgi-server` on a monitoring server.

If you want to use `systemd` to deploy, please change the server address in the service file provided here.

# Usage

```bash
➜  server git:(main) curl http://127.0.0.1:7070/info
>> 2023-06-03 12:01:31 [watchcorgi]
+---------+------+------+-------------------------------------+------+-------------------+--------------+-----------+
|   name  |cpu[s]|cpu[u]|              gpu device             |gpu[u]|       gpu[m]      |   gpu user   |update time|
+---------+------+------+-------------------------------------+------+-------------------+--------------+-----------+
| chouniu | 0.0 %| 0.0 %|      A100-PCIE-40GB(460.106.00)     |  0 % |  0 MiB/40536 MiB  |     null     |  12:01:22 |
|         |      |      |      A100-PCIE-40GB(460.106.00)     | 17 % |  0 MiB/40536 MiB  |              |           |
+---------+------+------+-------------------------------------+------+-------------------+--------------+-----------+
|fenghuang| 0.0 %| 0.0 %|  NVIDIA GeForce RTX 3090(515.65.01) |  0 % |  2 MiB/24576 MiB  |   StainAtt   |  12:01:30 |
|         |      |      |  NVIDIA GeForce RTX 3090(515.65.01) | 91 % |12611 MiB/24576 MiB|              |           |
+---------+------+------+-------------------------------------+------+-------------------+--------------+-----------+
|   imp   | 0.0 %| 0.0 %|NVIDIA GeForce GTX 1080 Ti(530.30.02)|  0 % |  0 MiB/11264 MiB  |     null     |  12:01:24 |
|         |      |      |NVIDIA GeForce GTX 1080 Ti(530.30.02)|  1 % |  0 MiB/11264 MiB  |              |           |
+---------+------+------+-------------------------------------+------+-------------------+--------------+-----------+
| kunpeng | 0.0 %| 0.2 %|                                     |      |                   | driver failed|  12:01:25 |
+---------+------+------+-------------------------------------+------+-------------------+--------------+-----------+
|  peppa  | 0.0 %| 0.0 %|NVIDIA GeForce RTX 2080 Ti(530.30.02)|  0 % |  0 MiB/11264 MiB  |     null     |  12:01:20 |
+---------+------+------+-------------------------------------+------+-------------------+--------------+-----------+
|   rick  | 0.1 %| 0.0 %|         Quadro P5000(510.54)        | 100 %|16145 MiB/16384 MiB|      CNN     |  12:01:29 |
+---------+------+------+-------------------------------------+------+-------------------+--------------+-----------+
| shiyuan | 0.0 %| 0.0 %|      A100-PCIE-40GB(460.106.00)     |  0 % |39262 MiB/40536 MiB|    API-Net   |  12:01:28 |
|         |      |      |      A100-PCIE-40GB(460.106.00)     |  0 % |  3 MiB/40536 MiB  |              |           |
+---------+------+------+-------------------------------------+------+-------------------+--------------+-----------+
|  swift  | 0.0 %| 0.0 %|NVIDIA GeForce RTX 2080 Ti(510.47.03)|  0 % |  1 MiB/11264 MiB  |     null     |  12:01:26 |
|         |      |      |NVIDIA GeForce RTX 2080 Ti(510.47.03)|  0 % |  1 MiB/11264 MiB  |              |           |
+---------+------+------+-------------------------------------+------+-------------------+--------------+-----------+
|  xuanwu | 0.0 %| 0.0 %|  NVIDIA A100-PCIE-40GB(525.116.03)  | 83 % |18796 MiB/40960 MiB|OpenHGNN_final|  12:01:23 |
|         |      |      |                                     |      |                   |   StainAtt   |           |
+---------+------+------+-------------------------------------+------+-------------------+--------------+-----------+
|  yinhu  | 0.0 %| 0.0 %| NVIDIA GeForce RTX 3090(525.116.03) |  0 % |  0 MiB/24576 MiB  |     null     |  12:01:28 |
+---------+------+------+-------------------------------------+------+-------------------+--------------+-----------+
|  zhuque | 0.5 %| 4.2 %|   NVIDIA A100-PCIE-40GB(515.65.01)  | 91 % | 3671 MiB/40960 MiB|     liif     |  12:01:26 |
+---------+------+------+-------------------------------------+------+-------------------+--------------+-----------+
|  zishu  | 0.0 %| 0.0 %| NVIDIA GeForce RTX 4090(525.116.03) |  0 % |  0 MiB/24564 MiB  |     null     |  12:01:18 |
+---------+------+------+-------------------------------------+------+-------------------+--------------+-----------+
Powered by Rust
```