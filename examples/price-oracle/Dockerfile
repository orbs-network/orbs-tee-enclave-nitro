# Dockerfile for AWS Nitro Enclave - Price Oracle
FROM public.ecr.aws/amazonlinux/amazonlinux:2023

# Install minimal dependencies
RUN yum install -y ca-certificates && \
    yum clean all && \
    rm -rf /var/cache/yum

# Copy the compiled binary (build context must be repo root)
COPY examples/price-oracle/target/release/price-oracle-enclave /app/price-oracle-enclave

# Set working directory
WORKDIR /app

# Make binary executable
RUN chmod +x /app/price-oracle-enclave

# Run the enclave application
CMD ["/app/price-oracle-enclave"]
