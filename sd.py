import math
import matplotlib.pyplot as plt
import numpy as np

since_epoch = 1735686000
jd = 2440587.5 + since_epoch / 86400 + 100
c = (jd - 2451_545) / 36525

res = []

for i in range(365):
    res.append(-23.45 * math.cos(math.radians((360.0 / 365.0) * (i + 10.0))))

xpoints = np.array(range(1, 366))
ypoints = res

plt.plot(xpoints, ypoints)
plt.show()

# print(declination * (180 / math.pi))
