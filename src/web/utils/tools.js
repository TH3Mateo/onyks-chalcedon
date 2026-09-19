import { degrees, PDFDocument, rgb, StandardFonts, PageSizes } from 'pdf-lib';
import QRCode from 'qrcode'
import { DateTime } from "luxon";
import { saveBinaryFile } from './files.js';

export const dateUTCtoDestination = (dateStr, zone = 'Europe/Warsaw', locale = 'en') =>
{
    return DateTime.fromISO(dateStr).setZone(zone).setLocale(locale).toLocaleString({...DateTime.DATETIME_MED_WITH_SECONDS, hour12: false})
}

export class LabelsDoc
{
    constructor(labelsAmount)
    {
        this.columns = 2
        this.rows = 6
        this.spacing = 7
        this.normalSize = 8
        this.bigSize = 12
        this.labelsAmount = labelsAmount
        this.width = PageSizes.A4[0]
        this.height = PageSizes.A4[1]
        this.widthStep = this.width / this.columns
        this.heightStep = this.height / this.rows
    }

    async addLogo(logoUrl)
    {
        this.logoBytes = await fetch(logoUrl).then((res) => res.arrayBuffer())
        this.logoImg = await this.pdf.embedPng(this.logoBytes)
        this.logoDims = await this.logoImg.scale(0.01)
    }

    async init()
    {
        this.pdf = await PDFDocument.create()
        for(let i=0; i<Math.ceil(this.labelsAmount/(this.columns*this.rows)); i++)
        {
            await this.pdf.addPage(PageSizes.A4)
        }
        this.pages = await this.pdf.getPages()
    }

    drawBorders()
    {
        for(let i=0; i<this.labelsAmount; i++)
        {
            this.__drawBorder(i)
        }
    }

    async __drawBorder(index)
    {
        let settings = 
        {
            thickness: 1,
            color: rgb(0, 0, 0),
            opacity: 0.5,
        }

        let col = index % this.columns
        let row = Math.floor((index % (this.rows * this.columns)) / 2)
        let page = Math.floor(index / (this.rows*this.columns))

        settings.start = { x: (this.width/2)*col, y: this.height - this.heightStep*(row + 1) }
        settings.end = { x: col == 0? this.width/2:this.width, y: this.height - this.heightStep*(row + 1) }

        await this.pages[page].drawLine(settings)

        if(col == 0)
        {
            settings.start = { x: this.width/2, y: this.height - this.heightStep*(row) }
            settings.end = { x: this.width/2, y: this.height - this.heightStep*(row + 1) }
            await this.pages[page].drawLine(settings)
        }
    }

    async drawData(index, data)
    {
        let col = index % this.columns
        let row = Math.floor((index % (this.rows * this.columns)) / 2)
        let page = Math.floor(index / (this.rows*this.columns))

        let positionLift = this.height - this.spacing - this.normalSize - this.heightStep * row

        this.pages[page].drawText(`UUID:\t${data.uuid}`, 
        {
            x: this.spacing + col*(this.width/2),
            y: positionLift,
            size: this.normalSize,
            color: rgb(0, 0, 0),
            rotate: degrees(0),
            lineHeight: this.normalSize
        })

        positionLift += -this.spacing - this.bigSize

        this.pages[page].drawText(`Part Name:\t${data.partName}`, 
        {
            x: this.spacing + col*(this.width/2),
            y: positionLift,
            size: this.bigSize,
            color: rgb(0, 0, 0),
            rotate: degrees(0),
            lineHeight: this.normalSize
        })

        positionLift += -this.spacing - this.bigSize

        this.pages[page].drawText(`Manufacturer:\t${data.manufacturer}`, 
        {
            x: this.spacing + col*(this.width/2),
            y: positionLift,
            size: this.normalSize,
            color: rgb(0, 0, 0),
            rotate: degrees(0),
            lineHeight: this.normalSize
        })

        positionLift += -this.spacing - this.normalSize

        this.pages[page].drawText(`Value:\t${data.value}`, 
        {
            x: this.spacing + col*(this.width/2),
            y: positionLift,
            size: this.normalSize,
            color: rgb(0, 0, 0),
            rotate: degrees(0),
            lineHeight: this.normalSize
        })

        positionLift += -this.spacing - this.normalSize

        this.pages[page].drawText(`Created at:\t${data.createdAt}`, 
        {
            x: this.spacing + col*(this.width/2),
            y: positionLift,
            size: this.normalSize,
            color: rgb(0, 0, 0),
            rotate: degrees(0),
            lineHeight: this.normalSize
        })

        positionLift += -this.spacing - this.normalSize

        this.pages[page].drawText(`Description:\t${data.description}`, 
        {
            x: this.spacing + col*(this.width/2),
            y: positionLift,
            size: this.normalSize,
            color: rgb(0, 0, 0),
            rotate: degrees(0),
            maxWidth: 200,
            lineHeight: this.normalSize
        })
    }

    async drawQR(index, uuid)
    {
        let col = index % this.columns
        let row = Math.floor((index % (this.rows * this.columns)) / 2)
        let page = Math.floor(index / (this.rows*this.columns))

        let qr = await QRCode.toDataURL(uuid)
        const qrImage = await this.pdf.embedPng(qr)
        const qrDims = await qrImage.scale(0.5)

        await this.pages[page].drawImage(qrImage, 
        {
            x: (this.width/2)*(col + 1) - qrDims.width - this.spacing + 5,
            y: this.height + this.spacing - (row + 1)*this.heightStep + 58,
            width: qrDims.width,
            height: qrDims.height,
        })
    }

    async drawLogo(index)
    {
        let col = index % this.columns
        let row = Math.floor((index % (this.rows * this.columns)) / 2)
        let page = Math.floor(index / (this.rows*this.columns))

        await this.pages[page].drawImage(this.logoImg, 
        {
            x: (this.width/2)*(col + 1) - this.logoDims.width - 3*this.spacing,
            y: this.height + 2*this.spacing - (row + 1)*this.heightStep + 95,
            width: this.logoDims.width,
            height: this.logoDims.height,
        })
    }

    async finish()
    {
        const pdfBytes = await this.pdf.save()
        const name = `onyks_bloodstone_label_${DateTime.now().setZone('Europe/Warsaw').toFormat('yyyy-MM-dd_HH-mm-ss')}.pdf`;
        await saveBinaryFile(pdfBytes, name, 'PDF', ['pdf']);
    }
}