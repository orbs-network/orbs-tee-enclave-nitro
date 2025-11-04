# Dockerfile for AWS Nitro Enclave - Price Oracle
FROM public.ecr.aws/amazonlinux/amazonlinux:2023

# Install minimal dependencies including socat for vsock DNS forwarding
RUN yum install -y ca-certificates socat iproute && \
    yum clean all && \
    rm -rf /var/cache/yum

# Copy the compiled binary (build context must be repo root)
COPY examples/price-oracle/target/release/price-oracle-enclave /app/price-oracle-enclave

# Set working directory
WORKDIR /app

# Make binary executable
RUN chmod +x /app/price-oracle-enclave

# Create startup script that sets up DNS forwarding and runs the app
RUN echo '#!/bin/sh' > /app/start.sh && \
    echo '# Configure DNS to localhost' >> /app/start.sh && \
    echo 'echo "nameserver 127.0.0.1" > /etc/resolv.conf' >> /app/start.sh && \
    echo '# Start DNS forwarder (vsock port 53 on parent to local UDP port 53)' >> /app/start.sh && \
    echo 'socat UDP4-LISTEN:53,fork,reuseaddr VSOCK-CONNECT:3:53 &' >> /app/start.sh && \
    echo '# Give DNS forwarder time to start' >> /app/start.sh && \
    echo 'sleep 2' >> /app/start.sh && \
    echo '# Run the price oracle application' >> /app/start.sh && \
    echo 'exec /app/price-oracle-enclave' >> /app/start.sh && \
    chmod +x /app/start.sh

# Run the startup script
CMD ["/app/start.sh"]
