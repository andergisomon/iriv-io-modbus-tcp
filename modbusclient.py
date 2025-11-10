# Sample client to quickly test the firmware

from pymodbus.client import ModbusTcpClient

c = ModbusTcpClient('172.30.40.36')
c.connect()

# print(hex(c.read_input_registers(0x0f00, count=1).registers[0])) # Should get 0x494f
# print(hex(c.read_input_registers(0x0f01, count=1).registers[0])) # Should get 0x4300

# print(c.read_input_registers(0x0f10, count=1).registers[0]) # Should get 0
# print(c.read_input_registers(0x0f11, count=1).registers[0]) # Should get 1
# print(c.read_input_registers(0x0f12, count=1).registers[0]) # Should get 0

# c.write_coil(0x0100, True)
# c.write_coil(0x0101, True)
# c.write_coil(0x0102, True)
# c.write_coil(0x0103, True)

# c.write_coil(0x0100, False)
# c.write_coil(0x0101, False)
# c.write_coil(0x0102, False)
# c.write_coil(0x0103, False)

# print(c.read_discrete_inputs(0x0000, count=1).bits)

c.close()
