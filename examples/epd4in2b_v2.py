# *****************************************************************************
# * | File        :	  epd4in2bc.py
# * | Author      :   Waveshare team
# * | Function    :   Electronic paper driver
# * | Info        :
# *----------------
# * | This version:   V4.1
# * | Date        :   2022-08-10
# # | Info        :   python demo
# -----------------------------------------------------------------------------
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documnetation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to  whom the Software is
# furished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in
# all copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS OR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
# THE SOFTWARE.
#

import logging
from . import epdconfig

# Display resolution
EPD_WIDTH       = 400
EPD_HEIGHT      = 300

logger = logging.getLogger(__name__)

class EPD:
    def __init__(self):
        self.reset_pin = epdconfig.RST_PIN
        self.dc_pin = epdconfig.DC_PIN
        self.busy_pin = epdconfig.BUSY_PIN
        self.cs_pin = epdconfig.CS_PIN
        self.width = EPD_WIDTH
        self.height = EPD_HEIGHT
        self.flag = 0
        
        if (epdconfig.module_init(cleanup=True) != 0):
            return -1
        

    # Hardware reset
    def reset(self):
        epdconfig.digital_write(self.reset_pin, 1)
        epdconfig.delay_ms(200) 
        epdconfig.digital_write(self.reset_pin, 0)
        epdconfig.delay_ms(5)
        epdconfig.digital_write(self.reset_pin, 1)
        epdconfig.delay_ms(200)   

    def send_command(self, command):
        epdconfig.digital_write(self.dc_pin, 0)
        epdconfig.digital_write(self.cs_pin, 0)
        epdconfig.DEV_SPI_write(command)
        epdconfig.digital_write(self.cs_pin, 1)

    def send_data(self, data):
        epdconfig.digital_write(self.dc_pin, 1)
        epdconfig.digital_write(self.cs_pin, 0)
        epdconfig.DEV_SPI_write(data)
        epdconfig.digital_write(self.cs_pin, 1)

    # send a lot of data   
    def send_data2(self, data):
        epdconfig.digital_write(self.dc_pin, 1)
        epdconfig.digital_write(self.cs_pin, 0)
        epdconfig.spi_writebyte2(data)
        epdconfig.digital_write(self.cs_pin, 1)
        
    def ReadBusy(self):
        logger.debug("e-Paper busy")
        if(self.flag == 1):
            while(epdconfig.digital_read(self.busy_pin) == 1): 
                epdconfig.delay_ms(100) 
        
        else:
            while(epdconfig.digital_read(self.busy_pin) == 0): 
                epdconfig.delay_ms(100) 
        logger.debug("e-Paper busy release")

    def TurnOnDisplay(self):
        if(self.flag == 1):
            self.send_command(0x22)
            self.send_data(0xF7)	
            self.send_command(0x20)
            self.ReadBusy()
        
        else:
            self.send_command(0x12)
            epdconfig.delay_ms(100) 
            self.ReadBusy()
            
    def init(self):
        i = 0x00
        self.reset()
        self.send_command(0x2F)
        epdconfig.delay_ms(100)
        epdconfig.digital_write(self.dc_pin, 1)
        epdconfig.digital_write(self.cs_pin, 0) 
        i = epdconfig.DEV_SPI_read()
        epdconfig.digital_write(self.cs_pin, 1) 
        # print(i)

        if(i == 0x01):
            self.flag = 1
            self.ReadBusy()
            self.send_command(0x12)
            self.ReadBusy()

            self.send_command(0x3C)
            self.send_data(0x05)	

            self.send_command(0x18)
            self.send_data(0x80)	

            self.send_command(0x11)      
            self.send_data(0x03)

            self.send_command(0x44) 
            self.send_data(0x00)
            self.send_data(self.width//8-1)

            self.send_command(0x45)        
            self.send_data(0x00)
            self.send_data(0x00) 
            self.send_data((self.height-1)%256)    
            self.send_data((self.height-1)//256)

            self.send_command(0x4E)
            self.send_data(0x00)
            self.send_command(0x4F)  
            self.send_data(0x00)    
            self.send_data(0x00)
            self.ReadBusy()

        else:
            self.flag = 0
            self.send_command(0x04)  # POWER_ON
            self.ReadBusy()

            self.send_command(0x00)  # panel setting
            self.send_data(0x0f)
        
        return 0

    def getbuffer(self, image):
        # logger.debug("bufsiz = ",int(self.width/8) * self.height)
        buf = [0xFF] * (int(self.width/8) * self.height)
        image_monocolor = image.convert('1')
        imwidth, imheight = image_monocolor.size
        pixels = image_monocolor.load()
        # logger.debug("imwidth = %d, imheight = %d",imwidth,imheight)
        if(imwidth == self.width and imheight == self.height):
            logger.debug("Horizontal")
            for y in range(imheight):
                for x in range(imwidth):
                    # Set the bits for the column of pixels at the current position.
                    if pixels[x, y] == 0:
                        buf[int((x + y * self.width) / 8)] &= ~(0x80 >> (x % 8))
        elif(imwidth == self.height and imheight == self.width):
            logger.debug("Vertical")
            for y in range(imheight):
                for x in range(imwidth):
                    newx = y
                    newy = self.height - x - 1
                    if pixels[x, y] == 0:
                        buf[int((newx + newy*self.width) / 8)] &= ~(0x80 >> (y % 8))
        return buf

    def display(self, imageblack, imagered):
        high = self.height
        if( self.width % 8 == 0) :
            wide =  self.width // 8
        else :
            wide =  self.width // 8 + 1
        
        if(self.flag == 1):
            self.send_command(0x24)
            for j in range(0, high):
                for i in range(0, wide):
                    self.send_data(imageblack[i + j * wide]) 
                    
            self.send_command(0x26)
            for j in range(0, high):
                for i in range(0, wide):
                    self.send_data(~imagered[i + j * wide]) 
        
        else:
            self.send_command(0x10)
            for j in range(0, high):
                for i in range(0, wide):
                    self.send_data(imageblack[i + j * wide]) 
                    
            self.send_command(0x13)
            for j in range(0, high):
                for i in range(0, wide):
                    self.send_data(~imagered[i + j * wide]) 

        self.TurnOnDisplay()
        
    def Clear(self):
        high = self.height
        if( self.width % 8 == 0) :
            wide =  self.width // 8
        else :
            wide =  self.width // 8 + 1

        if(self.flag == 1):
            self.send_command(0x24)
            for j in range(0, high):
                for i in range(0, wide):
                    self.send_data(0xff) 
                    
            self.send_command(0x26)
            for j in range(0, high):
                for i in range(0, wide):
                    self.send_data(0x00) 
        
        else:
            self.send_command(0x10)
            for j in range(0, high):
                for i in range(0, wide):
                    self.send_data(0xff) 
                    
            self.send_command(0x13)
            for j in range(0, high):
                for i in range(0, wide):
                    self.send_data(0x00) 

        self.TurnOnDisplay()

    def sleep(self):
        if(self.flag == 1):
            self.send_command(0X10) 
            self.send_data(0x03)
        
        else:
            self.send_command(0X50) 
            self.send_data(0xf7)             
            self.send_command(0X02)
            self.ReadBusy() 
            self.send_command(0X07) 
            self.send_data(0xA5)
        
        epdconfig.delay_ms(2000)
        epdconfig.module_exit()
### END OF FILE ###
