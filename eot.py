import math
import matplotlib.pyplot as plt
import numpy as np

since_epoch = 1749146045
jd = 2440587.5 + since_epoch / 86400
c = (jd - 2451_545) / 36525

res = []
for i in range(365 * 4):
    # degrees land
    halfLife = 280.46645 + 36000.76983 * c + 0.0003032 * c * c  # L0
    anomaly = (
        357.52910 + 35999.05030 * c - 0.0001559 * c * c - 0.00000048 * c * c * c
    )  # M
    eccentricity = 0.016708617 - 0.000042037 * c - 0.0000001236 * c  # e
    obliquity = (23 + 26 / 60 + 21.448 / 3600) + (
        -(46.8150 / 3600) * c - (0.00059 / 3600) * c * c + (0.001813) * c * c * c
    )  # weird e - aprox obliquity

    # radians land
    y = math.tan(math.radians(obliquity) / 2) * math.tan(math.radians(obliquity) / 2)
    eot = (
        y * math.sin(2 * math.radians(halfLife))
        - 2 * eccentricity * math.sin(math.radians(anomaly))
        + 4
        * eccentricity
        * y
        * math.sin(math.radians(anomaly))
        * math.cos(2 * math.radians(halfLife))
        - 0.5 * y * y * math.sin(4 * math.radians(halfLife))
        - 1.25 * eccentricity * eccentricity * math.sin(2 * math.radians(anomaly))
    )

    # print("eot in radian: ", eot)
    eot = eot * (180 / math.pi) / 15
    res.append(eot)
    c += 1 / 36525
    # print("eot in minutes: ", eot * 60)
    # print(
    #     int(eot * 60),
    #     "min",
    #     abs(int(((eot * 60) - int(eot * 60)) * 60)),
    #     "s",
    # )

xpoints = np.array(range(1, 365 * 4 + 1))
ypoints = res

plt.plot(xpoints, ypoints)
plt.show()
