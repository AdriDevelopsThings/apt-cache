FROM ubuntu

RUN sed -i 's/http:\/\/ports.ubuntu.com\/ubuntu-ports\//http:\/\/apt-cache\/ubuntu-ports\//' /etc/apt/sources.list
ADD ./ubuntu_update.sh /ubuntu_update.sh
RUN chmod a+x /ubuntu_update.sh

CMD ["/ubuntu_update.sh"]