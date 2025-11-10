# Sample client to quickly test the firmware

from pymodbus.client import ModbusTcpClient

c = ModbusTcpClient('172.30.40.36')
c.connect()

# print(hex(c.read_input_registers(0x0f00, count=1).registers[0])) # Should get 0x494f
# print(hex(c.read_input_registers(0x0f01, count=1).registers[0])) # Should get 0x4300

# c.write_coil(0x0100, True)
# c.write_coil(0x0101, True)
# c.write_coil(0x0102, True)
# c.write_coil(0x0103, True)

# c.write_coil(0x0100, False)
# c.write_coil(0x0101, False)
# c.write_coil(0x0102, False)
# c.write_coil(0x0103, False)

c.close()
